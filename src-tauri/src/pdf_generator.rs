//! Motor minimalista y libre de dependencias pesadas para la generación de documentos PDF 1.4.
//!
//! Diseñado específicamente para **Tally — Generador de Presupuestos en PDF**.
//! Utiliza las fuentes estándar integradas del estándar PDF (`Helvetica` y `Helvetica-Bold`)
//! con codificación `WinAnsiEncoding` (Windows-1252) para soportar todos los caracteres del
//! español (á, é, í, ó, ú, ñ, Ñ, ¿, ¡, $, €, etc.) sin incrustar archivos TTF pesados.

use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// Datos de contacto de una parte (Profesional o Cliente).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyInfo {
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub email: String,
}

/// Ítem individual dentro de la lista de productos/servicios (entre 3 y 10 ítems).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
}

/// Estructura completa del presupuesto recibida desde el frontend o desde pruebas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotePayload {
    pub quote_number: String,
    pub issue_date: String,
    pub validity: String,
    pub currency: String,
    pub professional: PartyInfo,
    pub client: PartyInfo,
    pub items: Vec<QuoteItem>,
}

/// Resultado de validación y cálculo aritmético del presupuesto.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedQuote {
    pub payload: QuotePayload,
    pub subtotals: Vec<f64>,
    pub total: f64,
}

impl QuotePayload {
    /// Valida las reglas de negocio (entre 3 y 10 productos, campos obligatorios, valores no negativos)
    /// y calcula los subtotales y el total general.
    pub fn validate_and_calculate(self) -> Result<CalculatedQuote, String> {
        if self.quote_number.trim().is_empty() {
            return Err("El número o código del presupuesto es obligatorio.".into());
        }
        if self.issue_date.trim().is_empty() {
            return Err("La fecha de emisión es obligatoria.".into());
        }
        if self.validity.trim().is_empty() {
            return Err("La vigencia del presupuesto es obligatoria.".into());
        }

        // Validar datos del profesional
        if self.professional.first_name.trim().is_empty()
            || self.professional.last_name.trim().is_empty()
            || self.professional.phone.trim().is_empty()
            || self.professional.email.trim().is_empty()
        {
            return Err(
                "Todos los datos del profesional (nombre, apellido, teléfono y correo) son obligatorios."
                    .into(),
            );
        }

        // Validar datos del cliente
        if self.client.first_name.trim().is_empty()
            || self.client.last_name.trim().is_empty()
            || self.client.phone.trim().is_empty()
            || self.client.email.trim().is_empty()
        {
            return Err(
                "Todos los datos del cliente (nombre, apellido, teléfono y correo) son obligatorios."
                    .into(),
            );
        }

        // Regla estricta: entre 3 y 10 productos
        let count = self.items.len();
        if !(3..=10).contains(&count) {
            return Err(format!(
                "La lista de productos debe contener entre 3 y 10 ítems (actualmente tiene {}).",
                count
            ));
        }

        let mut subtotals = Vec::with_capacity(count);
        let mut total = 0.0_f64;

        for (idx, item) in self.items.iter().enumerate() {
            if item.description.trim().is_empty() {
                return Err(format!(
                    "El producto #{} debe tener una descripción o nombre.",
                    idx + 1
                ));
            }
            if !item.quantity.is_finite() || item.quantity <= 0.0 {
                return Err(format!(
                    "La cantidad del producto #{} debe ser mayor a cero.",
                    idx + 1
                ));
            }
            if !item.unit_price.is_finite() || item.unit_price < 0.0 {
                return Err(format!(
                    "El precio unitario del producto #{} no puede ser negativo.",
                    idx + 1
                ));
            }
            let sub = (item.quantity * item.unit_price * 100.0).round() / 100.0;
            subtotals.push(sub);
            total += sub;
        }

        total = (total * 100.0).round() / 100.0;

        Ok(CalculatedQuote {
            payload: self,
            subtotals,
            total,
        })
    }
}

/// Convierte una cadena UTF-8 a una secuencia escapada en octal/ASCII compatible con `WinAnsiEncoding` (Windows-1252) de PDF.
fn pdf_escape_winansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 16);
    for ch in text.chars() {
        let code = ch as u32;
        let byte: u8 = match ch {
            '€' => 0x80,
            '—' => 0x97,
            '–' => 0x96,
            '•' => 0x95,
            '“' | '”' => b'"',
            '‘' | '’' => b'\'',
            _ if (0x20..=0x7E).contains(&code) => code as u8,
            _ if (0xA0..=0xFF).contains(&code) => code as u8,
            _ => b'?',
        };

        match byte {
            b'(' => out.push_str("\\("),
            b')' => out.push_str("\\)"),
            b'\\' => out.push_str("\\\\"),
            0x20..=0x7E => out.push(byte as char),
            _ => {
                let _ = write!(out, "\\{:03o}", byte);
            }
        }
    }
    out
}

/// Formatea un número decimal como moneda con separador de miles (`.`) y decimales (`,`).
pub fn format_currency(amount: f64, symbol: &str) -> String {
    let abs_val = amount.abs();
    let cents = ((abs_val * 100.0).round() as u64) % 100;
    let whole = (abs_val * 100.0).round() as u64 / 100;

    let whole_str = whole.to_string();
    let mut formatted_whole = String::new();
    for (i, c) in whole_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            formatted_whole.push('.');
        }
        formatted_whole.push(c);
    }
    let final_whole: String = formatted_whole.chars().rev().collect();
    let sym = if symbol.trim().is_empty() {
        "$"
    } else {
        symbol.trim()
    };

    if amount < 0.0 {
        format!("-{} {},{:02}", sym, final_whole, cents)
    } else {
        format!("{} {},{:02}", sym, final_whole, cents)
    }
}

/// Formatea una cantidad (sin decimales si es entero exacto, o hasta 2 decimales si tiene fracción).
fn format_quantity(qty: f64) -> String {
    if (qty - qty.round()).abs() < 1e-6 {
        format!("{}", qty.round() as i64)
    } else {
        format!("{:.2}", qty).replace('.', ",")
    }
}

/// Trunca un texto a un máximo de caracteres para mantener la maquetación limpia en la tabla.
fn truncate_text(text: &str, max_chars: usize) -> String {
    let clean = text.trim();
    if clean.chars().count() <= max_chars {
        clean.to_string()
    } else {
        let mut s: String = clean.chars().take(max_chars.saturating_sub(1)).collect();
        s.push('…');
        s
    }
}

/// Estima el ancho en puntos de una cadena en Helvetica / Helvetica-Bold para alineación derecha.
fn estimate_text_width(text: &str, font_size: f64) -> f64 {
    let mut units = 0.0;
    for ch in text.chars() {
        units += match ch {
            'i' | 'l' | 'I' | '.' | ',' | ':' | ';' | '!' | '|' | ' ' => 0.28,
            'f' | 'j' | 'r' | 't' | '(' | ')' | '-' => 0.35,
            'm' | 'w' | 'M' | 'W' | '@' | '%' => 0.82,
            'A'..='Z' => 0.68,
            '0'..='9' | '$' => 0.56,
            _ => 0.53,
        };
    }
    units * font_size
}

/// Construye el archivo PDF 1.4 binario en memoria con diseño editorial limpio y profesional (Tamaño A4: 595.28 x 841.89 pt).
pub fn render_quote_pdf(calc: &CalculatedQuote) -> Result<Vec<u8>, String> {
    let q = &calc.payload;
    let mut stream = String::with_capacity(8192);

    // Helper closures for drawing primitives in PDF content stream
    let draw_rect_fill = |s: &mut String, x: f64, y: f64, w: f64, h: f64, r: f64, g: f64, b: f64| {
        let _ = writeln!(s, "{:.3} {:.3} {:.3} rg {:.2} {:.2} {:.2} {:.2} re f", r, g, b, x, y, w, h);
    };

    let draw_rect_stroke =
        |s: &mut String, x: f64, y: f64, w: f64, h: f64, r: f64, g: f64, b: f64, lw: f64| {
            let _ = writeln!(
                s,
                "{:.2} w {:.3} {:.3} {:.3} RG {:.2} {:.2} {:.2} {:.2} re S",
                lw, r, g, b, x, y, w, h
            );
        };

    let draw_line =
        |s: &mut String, x1: f64, y1: f64, x2: f64, y2: f64, r: f64, g: f64, b: f64, lw: f64| {
            let _ = writeln!(
                s,
                "{:.2} w {:.3} {:.3} {:.3} RG {:.2} {:.2} m {:.2} {:.2} l S",
                lw, r, g, b, x1, y1, x2, y2
            );
        };

    let draw_text = |s: &mut String,
                     font: &str,
                     size: f64,
                     x: f64,
                     y: f64,
                     r: f64,
                     g: f64,
                     b: f64,
                     text: &str| {
        let escaped = pdf_escape_winansi(text);
        let _ = writeln!(
            s,
            "BT /{} {:.1} Tf {:.3} {:.3} {:.3} rg 1 0 0 1 {:.2} {:.2} Tm ({}) Tj ET",
            font, size, r, g, b, x, y, escaped
        );
    };

    let draw_text_right = |s: &mut String,
                           font: &str,
                           size: f64,
                           right_x: f64,
                           y: f64,
                           r: f64,
                           g: f64,
                           b: f64,
                           text: &str| {
        let w = estimate_text_width(text, size);
        let x = (right_x - w).max(40.0);
        draw_text(s, font, size, x, y, r, g, b, text);
    };

    // Paleta Editorial Profesional:
    // Navy Primario: #1D3557 -> (0.114, 0.208, 0.341)
    // Terracota Acento: #E63946 -> (0.902, 0.224, 0.275)
    // Gris Pizarra Texto: #1E293B -> (0.118, 0.161, 0.231)
    // Gris Secundario: #64748B -> (0.392, 0.455, 0.545)
    // Fondo Tarjeta Suave: #F8FAFC -> (0.973, 0.980, 0.988)
    // Borde Suave: #CBD5E1 -> (0.796, 0.835, 0.882)

    // 1. Franja superior de acento y Cabecera Principal
    draw_rect_fill(&mut stream, 0.0, 833.89, 595.28, 8.0, 0.902, 0.224, 0.275);
    draw_rect_fill(&mut stream, 40.0, 745.0, 515.28, 68.0, 0.114, 0.208, 0.341);

    draw_text(
        &mut stream,
        "F2",
        22.0,
        58.0,
        782.0,
        1.0,
        1.0,
        1.0,
        "PRESUPUESTO COMERCIAL",
    );
    draw_text(
        &mut stream,
        "F1",
        10.0,
        58.0,
        762.0,
        0.820,
        0.875,
        0.937,
        "Documento generado localmente con Tally",
    );

    // Código de presupuesto y fecha alineados a la derecha dentro del bloque de cabecera
    let code_label = format!("N.° {}", truncate_text(&q.quote_number, 24));
    draw_text_right(&mut stream, "F2", 13.0, 538.0, 785.0, 1.0, 1.0, 1.0, &code_label);
    let date_label = format!("Fecha de emisión: {}", truncate_text(&q.issue_date, 20));
    draw_text_right(
        &mut stream,
        "F1",
        9.5,
        538.0,
        764.0,
        0.88,
        0.92,
        0.96,
        &date_label,
    );

    // 2. Banda informativa de Vigencia y Moneda
    draw_rect_fill(&mut stream, 40.0, 712.0, 515.28, 24.0, 0.945, 0.961, 0.976);
    draw_rect_stroke(&mut stream, 40.0, 712.0, 515.28, 24.0, 0.796, 0.835, 0.882, 0.6);

    let validity_str = format!("Vigencia del presupuesto: {}", truncate_text(&q.validity, 45));
    draw_text(&mut stream, "F2", 9.5, 52.0, 720.5, 0.114, 0.208, 0.341, &validity_str);

    let currency_str = format!("Moneda: {}", truncate_text(&q.currency, 10));
    draw_text_right(&mut stream, "F2", 9.5, 542.0, 720.5, 0.114, 0.208, 0.341, &currency_str);

    // 3. Tarjetas de Datos: Profesional (Izquierda) y Cliente (Derecha)
    // Tarjeta Profesional
    draw_rect_fill(&mut stream, 40.0, 598.0, 250.0, 102.0, 0.973, 0.980, 0.988);
    draw_rect_stroke(&mut stream, 40.0, 598.0, 250.0, 102.0, 0.796, 0.835, 0.882, 0.7);
    draw_rect_fill(&mut stream, 40.0, 678.0, 250.0, 22.0, 0.271, 0.482, 0.616);
    draw_text(
        &mut stream,
        "F2",
        9.5,
        50.0,
        685.5,
        1.0,
        1.0,
        1.0,
        "DATOS DEL PROFESIONAL",
    );

    let prof_full = format!(
        "{} {}",
        q.professional.first_name.trim(),
        q.professional.last_name.trim()
    );
    draw_text(
        &mut stream,
        "F2",
        11.0,
        50.0,
        658.0,
        0.118,
        0.161,
        0.231,
        &truncate_text(&prof_full, 36),
    );
    draw_text(
        &mut stream,
        "F1",
        9.5,
        50.0,
        638.0,
        0.392,
        0.455,
        0.545,
        &format!("Tel: {}", truncate_text(&q.professional.phone, 32)),
    );
    draw_text(
        &mut stream,
        "F1",
        9.5,
        50.0,
        618.0,
        0.392,
        0.455,
        0.545,
        &format!("Email: {}", truncate_text(&q.professional.email, 34)),
    );

    // Tarjeta Cliente
    draw_rect_fill(&mut stream, 305.28, 598.0, 250.0, 102.0, 0.973, 0.980, 0.988);
    draw_rect_stroke(&mut stream, 305.28, 598.0, 250.0, 102.0, 0.796, 0.835, 0.882, 0.7);
    draw_rect_fill(&mut stream, 305.28, 678.0, 250.0, 22.0, 0.271, 0.482, 0.616);
    draw_text(
        &mut stream,
        "F2",
        9.5,
        315.28,
        685.5,
        1.0,
        1.0,
        1.0,
        "DATOS DEL CLIENTE",
    );

    let client_full = format!(
        "{} {}",
        q.client.first_name.trim(),
        q.client.last_name.trim()
    );
    draw_text(
        &mut stream,
        "F2",
        11.0,
        315.28,
        658.0,
        0.118,
        0.161,
        0.231,
        &truncate_text(&client_full, 36),
    );
    draw_text(
        &mut stream,
        "F1",
        9.5,
        315.28,
        638.0,
        0.392,
        0.455,
        0.545,
        &format!("Tel: {}", truncate_text(&q.client.phone, 32)),
    );
    draw_text(
        &mut stream,
        "F1",
        9.5,
        315.28,
        618.0,
        0.392,
        0.455,
        0.545,
        &format!("Email: {}", truncate_text(&q.client.email, 34)),
    );

    // 4. Tabla de Productos / Servicios (entre 3 y 10 ítems)
    let table_top = 576.0;
    let header_h = 26.0;
    let row_h = 26.0;

    // Cabecera de tabla
    draw_rect_fill(
        &mut stream,
        40.0,
        table_top - header_h,
        515.28,
        header_h,
        0.114,
        0.208,
        0.341,
    );

    let header_text_y = table_top - 17.0;
    draw_text(&mut stream, "F2", 9.5, 48.0, header_text_y, 1.0, 1.0, 1.0, "#");
    draw_text(
        &mut stream,
        "F2",
        9.5,
        75.0,
        header_text_y,
        1.0,
        1.0,
        1.0,
        "DESCRIPCIÓN DEL PRODUCTO / SERVICIO",
    );
    draw_text_right(
        &mut stream,
        "F2",
        9.5,
        365.0,
        header_text_y,
        1.0,
        1.0,
        1.0,
        "CANT.",
    );
    draw_text_right(
        &mut stream,
        "F2",
        9.5,
        455.0,
        header_text_y,
        1.0,
        1.0,
        1.0,
        "PRECIO UNIT.",
    );
    draw_text_right(
        &mut stream,
        "F2",
        9.5,
        545.0,
        header_text_y,
        1.0,
        1.0,
        1.0,
        "SUBTOTAL",
    );

    let mut current_y = table_top - header_h;

    for (idx, item) in q.items.iter().enumerate() {
        let row_bottom = current_y - row_h;
        // Filas cebreadas suaves para facilitar lectura
        if idx % 2 == 1 {
            draw_rect_fill(
                &mut stream,
                40.0,
                row_bottom,
                515.28,
                row_h,
                0.973,
                0.980,
                0.988,
            );
        }
        draw_line(
            &mut stream,
            40.0,
            row_bottom,
            555.28,
            row_bottom,
            0.85,
            0.88,
            0.92,
            0.5,
        );

        let text_y = row_bottom + 8.5;
        let num_str = format!("{:02}", idx + 1);
        draw_text(
            &mut stream,
            "F2",
            9.5,
            48.0,
            text_y,
            0.392,
            0.455,
            0.545,
            &num_str,
        );

        let desc = truncate_text(&item.description, 48);
        draw_text(
            &mut stream,
            "F1",
            10.0,
            75.0,
            text_y,
            0.118,
            0.161,
            0.231,
            &desc,
        );

        let qty_str = format_quantity(item.quantity);
        draw_text_right(
            &mut stream,
            "F1",
            10.0,
            365.0,
            text_y,
            0.118,
            0.161,
            0.231,
            &qty_str,
        );

        let unit_str = format_currency(item.unit_price, &q.currency);
        draw_text_right(
            &mut stream,
            "F1",
            10.0,
            455.0,
            text_y,
            0.118,
            0.161,
            0.231,
            &unit_str,
        );

        let sub_str = format_currency(calc.subtotals[idx], &q.currency);
        draw_text_right(
            &mut stream,
            "F2",
            10.0,
            545.0,
            text_y,
            0.118,
            0.161,
            0.231,
            &sub_str,
        );

        current_y = row_bottom;
    }

    // Contorno exterior de la tabla
    let table_height = table_top - current_y;
    draw_rect_stroke(
        &mut stream,
        40.0,
        current_y,
        515.28,
        table_height,
        0.796,
        0.835,
        0.882,
        0.8,
    );

    // 5. Bloque de Total General
    let total_box_top = current_y - 16.0;
    let total_box_h = 38.0;
    let total_box_bottom = total_box_top - total_box_h;

    draw_rect_fill(
        &mut stream,
        335.28,
        total_box_bottom,
        220.0,
        total_box_h,
        0.114,
        0.208,
        0.341,
    );
    draw_text(
        &mut stream,
        "F2",
        12.0,
        348.0,
        total_box_bottom + 14.0,
        1.0,
        1.0,
        1.0,
        "TOTAL",
    );

    let grand_total_str = format_currency(calc.total, &q.currency);
    draw_text_right(
        &mut stream,
        "F2",
        14.0,
        545.0,
        total_box_bottom + 13.5,
        1.0,
        1.0,
        1.0,
        &grand_total_str,
    );

    // 6. Pie de página formal
    draw_line(&mut stream, 40.0, 68.0, 555.28, 68.0, 0.796, 0.835, 0.882, 0.7);
    draw_text(
        &mut stream,
        "F1",
        8.5,
        40.0,
        52.0,
        0.392,
        0.455,
        0.545,
        &format!(
            "Presupuesto {} — Validez: {}. Los precios expresados corresponden al total final acordado.",
            truncate_text(&q.quote_number, 20),
            truncate_text(&q.validity, 35)
        ),
    );
    draw_text(
        &mut stream,
        "F1",
        8.0,
        40.0,
        38.0,
        0.55,
        0.60,
        0.68,
        "Generado de forma 100% local con Tally (Desarrollado por Victoria Sanjurjo).",
    );

    // Ensamblado de objetos PDF 1.4 con tabla XREF exacta
    let stream_bytes = stream.as_bytes();
    let mut pdf = Vec::with_capacity(stream_bytes.len() + 2048);
    pdf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let mut offsets = Vec::with_capacity(6);

    // Obj 1: Catalog
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Obj 2: Pages
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // Obj 3: Page (A4: 595.28 x 841.89)
    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595.28 841.89] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> /Contents 6 0 R >>\nendobj\n",
    );

    // Obj 4: Font F1 (Helvetica WinAnsiEncoding)
    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    // Obj 5: Font F2 (Helvetica-Bold WinAnsiEncoding)
    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    // Obj 6: Content Stream
    offsets.push(pdf.len());
    let header_obj6 = format!("6 0 obj\n<< /Length {} >>\nstream\n", stream_bytes.len());
    pdf.extend_from_slice(header_obj6.as_bytes());
    pdf.extend_from_slice(stream_bytes);
    pdf.extend_from_slice(b"endstream\nendobj\n");

    // XREF
    let xref_offset = pdf.len();
    let xref_header = format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1);
    pdf.extend_from_slice(xref_header.as_bytes());
    for off in &offsets {
        let entry = format!("{:010} 00000 n \n", off);
        pdf.extend_from_slice(entry.as_bytes());
    }

    // Trailer
    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        offsets.len() + 1,
        xref_offset
    );
    pdf.extend_from_slice(trailer.as_bytes());

    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload(item_count: usize) -> QuotePayload {
        let mut items = Vec::new();
        for i in 1..=item_count {
            items.push(QuoteItem {
                description: format!("Servicio de desarrollo módulo {}", i),
                quantity: i as f64,
                unit_price: 1500.50,
            });
        }
        QuotePayload {
            quote_number: "PRES-0001".into(),
            issue_date: "06/11/2026".into(),
            validity: "15 días hábiles".into(),
            currency: "$".into(),
            professional: PartyInfo {
                first_name: "Victoria".into(),
                last_name: "Sanjurjo".into(),
                phone: "+54 11 5555-0101".into(),
                email: "victoria@ejemplo.com".into(),
            },
            client: PartyInfo {
                first_name: "Martín".into(),
                last_name: "Gómez".into(),
                phone: "+54 11 4444-0202".into(),
                email: "martin.gomez@cliente.com".into(),
            },
            items,
        }
    }

    #[test]
    fn rejects_fewer_than_three_items() {
        let payload = sample_payload(2);
        let err = payload.validate_and_calculate().unwrap_err();
        assert!(err.contains("entre 3 y 10"));
    }

    #[test]
    fn rejects_more_than_ten_items() {
        let payload = sample_payload(11);
        let err = payload.validate_and_calculate().unwrap_err();
        assert!(err.contains("entre 3 y 10"));
    }

    #[test]
    fn calculates_subtotals_and_total_accurately() {
        let payload = sample_payload(3);
        let calc = payload.validate_and_calculate().expect("Debe ser válido");
        assert_eq!(calc.subtotals, vec![1500.50, 3001.00, 4501.50]);
        assert_eq!(calc.total, 9003.00);
    }

    #[test]
    fn generates_valid_pdf_binary() {
        let payload = sample_payload(4);
        let calc = payload.validate_and_calculate().unwrap();
        let pdf_bytes = render_quote_pdf(&calc).expect("Debe renderizar PDF");
        assert!(pdf_bytes.starts_with(b"%PDF-1.4"));
        assert!(pdf_bytes.ends_with(b"%%EOF\n"));
    }
}

