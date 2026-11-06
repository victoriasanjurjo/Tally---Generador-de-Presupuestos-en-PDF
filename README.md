# Tally — Generador de Presupuestos en PDF

**Tally** es una aplicación nativa de escritorio para Windows desarrollada con **Rust + Tauri v2** y un frontend ligero en **HTML5, CSS3 y Vanilla JavaScript**, diseñada para que profesionales independientes, consultores y comercios generen presupuestos formales en formato **PDF** de manera rápida, clara y **100% local**.

> **Privacidad Garantizada:** Sin registro de cuentas, sin bases de datos en la nube, sin telemetría, sin recopilación de datos del equipo ni del usuario, y sin suscripciones ni pagos integrados.

---

## Características Principales

- **Metadatos de Emisión Autogenerados y Editables:** Asigna automáticamente la fecha actual del sistema (`DD/MM/AAAA`) y un código secuencial sugerido (`PRES-0001`), permitiendo editarlos libremente en cualquier momento.
- **Vigencia y Moneda Configurables:** Define el plazo de validez de la oferta (ej. *15 días hábiles*) y el símbolo monetario (`$`, `USD`, `ARS`, `€`).
- **Datos Completos del Profesional y del Cliente:** Campos estructurados y validados para **Nombre**, **Apellido**, **Número de Teléfono** y **Correo Electrónico** tanto del profesional emisor como del cliente destinatario.
- **Lista de Productos Controlada (entre 3 y 10 ítems):** Garantiza en todo momento un mínimo de **3** y un máximo de **10** productos o servicios, con botones intuitivos para agregar o quitar filas dentro de ese rango.
- **Cálculo Automático en Tiempo Real:** Los subtotales por línea (`Cantidad × Precio Unitario`) y el **Total General** del presupuesto se actualizan instantáneamente a medida que el profesional escribe.
- **Generación de PDF Profesional Nativa:** Motor PDF 1.4 escrito directamente en Rust con diseño editorial limpio (tamaño A4, tipografía vectorial nítida, soporte completo de caracteres en español y cuadro de diálogo nativo de Windows para elegir dónde guardar el archivo).
- **Empaquetado Oficial `.msix` Firmado:** Incluye el paquete instalador `outputs/Tally_1.0.0_x64.msix` firmado digitalmente con certificado de desarrollo a nombre de **Victoria Sanjurjo**, además del ejecutable portable `outputs/Tally_1.0.0_x64_Portable.exe`.

---

## Arquitectura y Árbol de Directorios

El proyecto respeta una estructura modular, limpia y completamente documentada:

```text
Tally - Generador de Presupuestos en PDF/
├── assets/                          # Recursos visuales e íconos de la aplicación y MSIX
│   ├── icon.ico                     # Ícono principal de ventana Windows (256x256)
│   ├── icon.png                     # Ícono maestro en alta resolución
│   ├── Square44x44Logo.png          # Activo visual MSIX (44x44)
│   ├── Square150x150Logo.png        # Activo visual MSIX (150x150)
│   ├── StoreLogo.png                # Activo visual MSIX Store (50x50)
│   └── Wide310x150Logo.png          # Activo visual MSIX Tile (310x150)
├── config/                          # Configuraciones locales, manifiestos y scripts de build
│   ├── app_config.json              # Parámetros base de la aplicación y banderas de privacidad
│   ├── AppxManifest.xml             # Manifiesto oficial de empaquetado Windows MSIX
│   ├── build_msix.ps1               # Script de creación de certificado, empaquetado y firma MSIX
│   └── generate_icons.ps1           # Generador reproducible de íconos PNG/ICO
├── docs/                            # Documentación técnica y decisiones de diseño
│   └── ARCHITECTURE.md              # Justificación de arquitectura, dependencias y motor PDF
├── outputs/                         # Salida de paquetes compilados y presupuestos PDF generados
│   ├── Presupuesto_Ejemplo_PRES-0001.pdf # Reporte/Presupuesto PDF generado de muestra
│   ├── Tally_1.0.0_x64.msix         # Instalador Windows MSIX firmado (Victoria Sanjurjo)
│   ├── Tally_1.0.0_x64_Portable.exe # Binario ejecutable nativo portable
│   └── VictoriaSanjurjo_TallyDev.cer# Certificado público para instalar el MSIX
├── src/                             # Código fuente del Frontend (Vanilla HTML5/CSS3/JS)
│   ├── index.html                   # Estructura semántica y formulario de presupuestos
│   ├── styles.css                   # Diseño editorial claro, :user-invalid y @container
│   └── main.js                      # Lógica reactiva de subtotales, validaciones e IPC Tauri
├── src-tauri/                       # Código fuente del Backend Nativo (Rust + Tauri v2)
│   ├── capabilities/
│   │   └── default.json             # Permisos mínimos de seguridad de Tauri v2
│   ├── icons/                       # Íconos enlazados al recurso del ejecutable
│   ├── src/
│   │   ├── main.rs                  # Punto de entrada Tauri, comandos IPC y diálogo de guardado
│   │   └── pdf_generator.rs         # Validador de negocio y motor nativo de renderizado PDF 1.4
│   ├── build.rs                     # Script de compilación de recursos Tauri
│   ├── Cargo.toml                   # Manifiesto de dependencias mínimas en Rust
│   └── tauri.conf.json              # Configuración de ventana, CSP estricto y bundle
├── tests/                           # Pruebas de validación estructural
│   └── validation_tests.rs          # Verificación de integridad de directorios y reglas
├── .gitignore                       # Exclusiones de control de versiones Git
├── LICENSE.txt                      # Licencia MIT vigente (Copyright 2026 Victoria Sanjurjo)
└── README.md                        # Guía general del proyecto y manual de ejecución
```

---

## Requisitos Previos (Para Compilar desde el Código Fuente)

1. **Sistema Operativo:** Windows 10 (versión 1809+) o Windows 11 (64 bits) con **Microsoft Edge WebView2 Runtime** (incluido por defecto en Windows).
2. **Rust Toolchain:** `rustc` y `cargo` (estable, edición 2021).
3. **Windows 10/11 SDK:** Herramientas `makeappx.exe` y `signtool.exe` (para regenerar el paquete `.msix`).

---

## Instrucciones Paso a Paso de Ejecución y Empaquetado

### Opción A: Usar directamente el Ejecutable o el Instalador `.msix` (Sin compilar)

1. **Ejecución inmediata (Portable):**
   Haz doble clic sobre `outputs\Tally_1.0.0_x64_Portable.exe`.
2. **Instalación mediante paquete `.msix`:**
   - Como el paquete está firmado con un certificado de desarrollo autofirmado con tu Publisher ID (`CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B`, nombre visible **Victoria Sanjurjo**), para instalarlo con doble clic en un equipo nuevo importa primero el certificado `outputs\VictoriaSanjurjo_TallyDev.cer` en el almacén **Entidades de certificación raíz de confianza** (o **Personas de confianza**) del equipo local.
   - Luego abre `outputs\Tally_1.0.0_x64.msix` y haz clic en **Instalar**.

### Opción B: Compilar y Probar desde el Código Fuente

1. **Ejecutar las pruebas unitarias del motor de presupuestos y PDF:**
   ```powershell
   cd src-tauri
   cargo test
   ```
2. **Compilar el binario optimizado de producción (`release`):**
   ```powershell
   cd src-tauri
   cargo build --release
   ```
   El ejecutable quedará generado en `src-tauri\target\release\tally.exe`.
3. **Generar un presupuesto PDF de muestra por consola en `outputs/`:**
   ```powershell
   .\src-tauri\target\release\tally.exe --generate-sample-pdf .\outputs\Presupuesto_Ejemplo_PRES-0001.pdf
   ```
4. **Empaquetar y firmar el instalador `.msix`:**
   Desde la raíz del proyecto, ejecuta:
   ```powershell
   powershell -ExecutionPolicy Bypass -File config\build_msix.ps1
   ```
   Esto generará `outputs\Tally_1.0.0_x64.msix` firmado digitalmente por **Victoria Sanjurjo**.

---

## Autoría y Licencia

Desarrollado por **Victoria Sanjurjo**. Distribuido bajo la licencia **MIT** (consultar archivo `LICENSE.txt`).

