/**
 * Tally — Generador de Presupuestos en PDF
 * Lógica principal del Frontend en Vanilla JavaScript (ES2023+)
 * 100% Local, sin telemetría, conectado al backend de Rust mediante Tauri v2 IPC.
 */

(() => {
  "use strict";

  const MIN_ITEMS = 3;
  const MAX_ITEMS = 10;

  const form = document.getElementById("quote-form");
  const tbody = document.getElementById("items-tbody");
  const btnAddItem = document.getElementById("btn-add-item");
  const btnDemoData = document.getElementById("btn-demo-data");
  const btnClearForm = document.getElementById("btn-clear-form");
  const btnGeneratePdf = document.getElementById("btn-generate-pdf");
  const itemsCounter = document.getElementById("items-counter");
  const grandTotalValue = document.getElementById("grand-total-value");
  const summaryValidityPreview = document.getElementById("summary-validity-preview");
  const statusBanner = document.getElementById("status-banner");

  const quoteNumberInput = document.getElementById("quote-number");
  const issueDateInput = document.getElementById("issue-date");
  const validityInput = document.getElementById("validity");
  const currencyInput = document.getElementById("currency");

  // 1. Utilidad de Fallback y Sincronización ARIA para :user-invalid (Modern Web Guidance)
  const UserInvalidFallback = (() => {
    const dirtyState = new WeakMap();

    const updateState = (input) => {
      if (!input || typeof input.checkValidity !== "function") return;
      const isValid = input.checkValidity();
      input.classList.toggle("user-invalid-fallback", !isValid);
      input.classList.toggle("user-valid-fallback", isValid);

      if (!isValid) {
        input.setAttribute("aria-invalid", "true");
      } else {
        input.removeAttribute("aria-invalid");
      }
    };

    const handleEvent = (event) => {
      const input = event.target;
      if (event.type === "reset") {
        const controls = input.elements || [];
        for (const control of controls) {
          dirtyState.delete(control);
          control.classList.remove("user-invalid-fallback", "user-valid-fallback");
          control.removeAttribute("aria-invalid");
        }
        return;
      }

      if (!input || typeof input.checkValidity !== "function") return;

      if (event.type === "input" || event.type === "change") {
        const state = dirtyState.get(input) || { hasInteracted: false, hasBlurred: false };
        state.hasInteracted = true;
        dirtyState.set(input, state);
        if (state.hasBlurred || input.hasAttribute("aria-invalid")) {
          updateState(input);
        }
      } else if (event.type === "blur") {
        const state = dirtyState.get(input) || { hasInteracted: false, hasBlurred: false };
        state.hasBlurred = true;
        dirtyState.set(input, state);
        if (state.hasInteracted) {
          updateState(input);
        }
      }
    };

    const init = (root = document) => {
      // Sincroniza siempre aria-invalid para lectores de pantalla y activa clases fallback si es necesario
      root.addEventListener("blur", handleEvent, true);
      root.addEventListener("input", handleEvent);
      root.addEventListener("change", handleEvent);
      root.addEventListener("reset", handleEvent, true);
    };

    const markAllChecked = (formEl) => {
      const inputs = formEl.querySelectorAll("input[required]");
      inputs.forEach((inp) => updateState(inp));
    };

    return { init, markAllChecked };
  })();

  UserInvalidFallback.init(form);

  // 2. Formateador numérico de moneda en tiempo real
  function formatCurrency(amount) {
    const sym = (currencyInput.value || "$").trim() || "$";
    const safeAmount = Number.isFinite(amount) ? amount : 0;
    const formatted = new Intl.NumberFormat("es-AR", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(safeAmount);
    return `${sym} ${formatted}`;
  }

  // 3. Mostrar mensajes de estado
  function showStatus(message, type = "success") {
    statusBanner.textContent = message;
    statusBanner.className = `status-banner ${type}`;
  }

  function hideStatus() {
    statusBanner.textContent = "";
    statusBanner.className = "status-banner hidden";
  }

  // 4. Gestión de filas de productos (entre 3 y 10)
  function createItemRow(index, initialData = null) {
    const tr = document.createElement("tr");
    tr.className = "item-row";

    const descVal = initialData ? initialData.description : "";
    const qtyVal = initialData ? initialData.quantity : "1";
    const priceVal = initialData ? initialData.unit_price : "";

    tr.innerHTML = `
      <th scope="row" class="row-num">${String(index).padStart(2, "0")}</th>
      <td>
        <div class="field">
          <label for="item-desc-${index}" class="sr-only">Descripción del producto ${index}</label>
          <input
            type="text"
            id="item-desc-${index}"
            class="item-desc"
            required
            maxlength="80"
            placeholder="Descripción del producto o servicio #${index}"
            value="${descVal}"
            aria-errormessage="item-desc-err-${index}"
          />
          <div id="item-desc-err-${index}" class="error-msg">
            <span aria-hidden="true">❌</span> Ingrese el nombre del producto.
          </div>
        </div>
      </td>
      <td>
        <div class="field">
          <label for="item-qty-${index}" class="sr-only">Cantidad del producto ${index}</label>
          <input
            type="number"
            id="item-qty-${index}"
            class="item-qty"
            required
            min="0.01"
            step="any"
            placeholder="1"
            value="${qtyVal}"
            aria-errormessage="item-qty-err-${index}"
          />
          <div id="item-qty-err-${index}" class="error-msg">
            <span aria-hidden="true">❌</span> Cantidad &gt; 0.
          </div>
        </div>
      </td>
      <td>
        <div class="field">
          <label for="item-price-${index}" class="sr-only">Precio unitario del producto ${index}</label>
          <input
            type="number"
            id="item-price-${index}"
            class="item-price"
            required
            min="0"
            step="0.01"
            placeholder="0.00"
            value="${priceVal}"
            aria-errormessage="item-price-err-${index}"
          />
          <div id="item-price-err-${index}" class="error-msg">
            <span aria-hidden="true">❌</span> Ingrese un precio válido.
          </div>
        </div>
      </td>
      <td class="subtotal-cell" data-subtotal="0">$ 0,00</td>
      <td class="action-cell">
        <button type="button" class="btn btn-remove" title="Quitar producto">
          Quitar
        </button>
      </td>
    `;

    const qtyInput = tr.querySelector(".item-qty");
    const priceInput = tr.querySelector(".item-price");
    const removeBtn = tr.querySelector(".btn-remove");

    qtyInput.addEventListener("input", recalculateAll);
    priceInput.addEventListener("input", recalculateAll);

    removeBtn.addEventListener("click", () => {
      const currentRows = tbody.querySelectorAll("tr.item-row");
      if (currentRows.length <= MIN_ITEMS) {
        showStatus(`El presupuesto debe tener como mínimo ${MIN_ITEMS} productos.`, "error");
        return;
      }
      tr.remove();
      renumberRows();
      recalculateAll();
    });

    return tr;
  }

  function renumberRows() {
    const rows = tbody.querySelectorAll("tr.item-row");
    const count = rows.length;

    rows.forEach((tr, idx) => {
      const num = idx + 1;
      const th = tr.querySelector(".row-num");
      if (th) th.textContent = String(num).padStart(2, "0");

      const desc = tr.querySelector(".item-desc");
      if (desc && !desc.value) {
        desc.placeholder = `Descripción del producto o servicio #${num}`;
      }

      const removeBtn = tr.querySelector(".btn-remove");
      if (removeBtn) {
        removeBtn.disabled = count <= MIN_ITEMS;
      }
    });

    itemsCounter.textContent = `${count} / ${MAX_ITEMS} productos`;
    btnAddItem.disabled = count >= MAX_ITEMS;
  }

  function recalculateAll() {
    const rows = tbody.querySelectorAll("tr.item-row");
    let grandTotal = 0;

    rows.forEach((tr) => {
      const qtyRaw = parseFloat(tr.querySelector(".item-qty")?.value || "0");
      const priceRaw = parseFloat(tr.querySelector(".item-price")?.value || "0");

      const qty = Number.isFinite(qtyRaw) && qtyRaw > 0 ? qtyRaw : 0;
      const price = Number.isFinite(priceRaw) && priceRaw >= 0 ? priceRaw : 0;

      const subtotal = Math.round(qty * price * 100) / 100;
      grandTotal += subtotal;

      const subCell = tr.querySelector(".subtotal-cell");
      if (subCell) {
        subCell.dataset.subtotal = String(subtotal);
        subCell.textContent = formatCurrency(subtotal);
      }
    });

    grandTotal = Math.round(grandTotal * 100) / 100;
    grandTotalValue.textContent = formatCurrency(grandTotal);
  }

  function initializeDefaultRows() {
    tbody.innerHTML = "";
    for (let i = 1; i <= MIN_ITEMS; i++) {
      tbody.appendChild(createItemRow(i));
    }
    renumberRows();
    recalculateAll();
  }

  // 5. Inicialización con metadatos del backend en Rust (Tauri v2 IPC)
  async function loadInitialMetadata() {
    const todayFallback = new Intl.DateTimeFormat("es-AR", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
    }).format(new Date());

    issueDateInput.value = todayFallback;

    const invoke = window.__TAURI__?.core?.invoke;
    if (typeof invoke === "function") {
      try {
        const meta = await invoke("get_initial_metadata");
        if (meta.suggested_number) quoteNumberInput.value = meta.suggested_number;
        if (meta.today_date) issueDateInput.value = meta.today_date;
        if (meta.default_validity) {
          validityInput.value = meta.default_validity;
          summaryValidityPreview.textContent = meta.default_validity;
        }
        if (meta.default_currency) {
          currencyInput.value = meta.default_currency;
        }
        recalculateAll();
      } catch (_err) {
        // Fallback silencioso local
      }
    }
  }

  // 6. Eventos de controles e interacciones
  btnAddItem.addEventListener("click", () => {
    hideStatus();
    const currentCount = tbody.querySelectorAll("tr.item-row").length;
    if (currentCount >= MAX_ITEMS) {
      showStatus(`Se alcanzó el límite máximo de ${MAX_ITEMS} productos.`, "error");
      return;
    }
    tbody.appendChild(createItemRow(currentCount + 1));
    renumberRows();
    recalculateAll();
  });

  currencyInput.addEventListener("input", recalculateAll);
  validityInput.addEventListener("input", () => {
    summaryValidityPreview.textContent = validityInput.value.trim() || "—";
  });

  btnDemoData.addEventListener("click", () => {
    hideStatus();
    document.getElementById("prof-first-name").value = "Victoria";
    document.getElementById("prof-last-name").value = "Sanjurjo";
    document.getElementById("prof-phone").value = "+54 11 5555-0100";
    document.getElementById("prof-email").value = "victoria.sanjurjo@profesional.com";

    document.getElementById("client-first-name").value = "Alejandro";
    document.getElementById("client-last-name").value = "Mendoza";
    document.getElementById("client-phone").value = "+54 11 4789-3321";
    document.getElementById("client-email").value = "amendoza@empresa-cliente.com";

    validityInput.value = "15 días hábiles";
    summaryValidityPreview.textContent = "15 días hábiles";
    currencyInput.value = "$";

    const demoItems = [
      {
        description: "Relevamiento técnico y diseño de arquitectura de software",
        quantity: "1",
        unit_price: "185000",
      },
      {
        description: "Desarrollo de módulos principales y lógica de negocio local",
        quantity: "2",
        unit_price: "210000",
      },
      {
        description: "Pruebas de calidad, empaquetado MSIX e implementación",
        quantity: "1",
        unit_price: "95000",
      },
    ];

    tbody.innerHTML = "";
    demoItems.forEach((item, i) => {
      tbody.appendChild(createItemRow(i + 1, item));
    });
    renumberRows();
    recalculateAll();
    showStatus("Datos de ejemplo cargados. Puede editarlos o generar el PDF directamente.", "success");
  });

  btnClearForm.addEventListener("click", () => {
    form.reset();
    hideStatus();
    initializeDefaultRows();
    loadInitialMetadata();
  });

  // 7. Envío del formulario y generación nativa del PDF
  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    hideStatus();

    UserInvalidFallback.markAllChecked(form);

    if (!form.checkValidity()) {
      const firstInvalid = form.querySelector(":invalid");
      if (firstInvalid) firstInvalid.focus();
      showStatus(
        "Por favor complete todos los campos obligatorios con el formato válido antes de generar el PDF.",
        "error"
      );
      return;
    }

    const rows = Array.from(tbody.querySelectorAll("tr.item-row"));
    if (rows.length < MIN_ITEMS || rows.length > MAX_ITEMS) {
      showStatus(`La lista de productos debe tener entre ${MIN_ITEMS} y ${MAX_ITEMS} ítems.`, "error");
      return;
    }

    const items = rows.map((tr) => ({
      description: tr.querySelector(".item-desc").value.trim(),
      quantity: parseFloat(tr.querySelector(".item-qty").value),
      unit_price: parseFloat(tr.querySelector(".item-price").value),
    }));

    const payload = {
      quote_number: quoteNumberInput.value.trim(),
      issue_date: issueDateInput.value.trim(),
      validity: validityInput.value.trim(),
      currency: currencyInput.value.trim() || "$",
      professional: {
        first_name: document.getElementById("prof-first-name").value.trim(),
        last_name: document.getElementById("prof-last-name").value.trim(),
        phone: document.getElementById("prof-phone").value.trim(),
        email: document.getElementById("prof-email").value.trim(),
      },
      client: {
        first_name: document.getElementById("client-first-name").value.trim(),
        last_name: document.getElementById("client-last-name").value.trim(),
        phone: document.getElementById("client-phone").value.trim(),
        email: document.getElementById("client-email").value.trim(),
      },
      items,
    };

    const invoke = window.__TAURI__?.core?.invoke;
    if (typeof invoke !== "function") {
      showStatus(
        "El puente nativo de Tauri no está disponible en este entorno de navegador externo.",
        "error"
      );
      return;
    }

    try {
      btnGeneratePdf.disabled = true;
      btnGeneratePdf.textContent = "Generando PDF...";

      const result = await invoke("generate_and_save_pdf", { payload });
      showStatus(
        `✔ Presupuesto generado exitosamente (${result.total_formatted}). Guardado en: ${result.saved_path}`,
        "success"
      );

      // Actualizar número sugerido para el próximo presupuesto
      const meta = await invoke("get_initial_metadata");
      if (meta?.suggested_number) {
        quoteNumberInput.value = meta.suggested_number;
      }
    } catch (err) {
      if (err === "CANCELLED") {
        showStatus("Guardado cancelado por el usuario.", "error");
      } else {
        showStatus(`Error al generar el PDF: ${err}`, "error");
      }
    } finally {
      btnGeneratePdf.disabled = false;
      btnGeneratePdf.textContent = "Generar y Descargar Presupuesto en PDF";
    }
  });

  // Inicialización al arrancar la ventana
  initializeDefaultRows();
  loadInitialMetadata();
})();

