Actúa como un Ingeniero de Software Experto en Rust, Tauri v2 y Desarrollo Mobile/Escritorio Multiplataforma.

Quiero construir una aplicación multiplataforma (Windows, macOS, iOS y Android) optimizada, ultraligera y de alto rendimiento utilizando Tauri v2. La aplicación es un Editor, Compresor, Divisor, Unidor y Procesador por lotes de archivos PDF, que además incluye capacidades de OCR.

Por decisión de diseño, hemos excluido el soporte para Linux para enfocarnos exclusivamente en los motores de OCR nativos y gratuitos de cada sistema operativo, evitando empaquetar modelos de IA pesados de terceros.

Por favor, ayúdame a inicializar el proyecto, diseñar la arquitectura de archivos y escribir los componentes clave basándote en los siguientes requerimientos tecnológicos y de negocio:

### 1. REQUERIMIENTOS DE BACKEND (Rust & APIs Nativas)
- **Manipulación de PDF:** Utiliza los crates `lopdf` para la lectura, división, unión y análisis estructural de archivos PDF, y `pdf-writer` para generar PDFs eficientemente y renderizar capas de texto invisibles sobre imágenes procesadas por OCR.
- **Procesamiento por Lotes y Paralelismo:** Implementa el crate `rayon` para paralelizar las operaciones de compresión, división, unión y OCR en múltiples hilos mediante iteradores paralelos (`par_iter`).
- **Arquitectura de OCR Nativo (Local y Gratuito):**
  - **Windows:** Implementa bindings con `windows-rs` para consumir la API nativa `Windows.Media.Ocr`.
  - **macOS / iOS:** Configura la app para invocar el framework nativo de Apple `Vision` (`VNRecognizeTextRequest`) mediante código Swift en el directorio nativo de Tauri v2 o bindings de Objective-C con el crate `objc2`.
  - **Android:** Configura llamadas a `ML Kit Text Recognition` de Google usando código nativo Kotlin en la sección móvil de Tauri.
- **Compresión de PDF:** Diseña una estrategia para extraer flujos de imágenes del PDF (usando `lopdf`), re-codificarlas a menor resolución/calidad (usando el crate `image`) y re-ensamblar el PDF reduciendo su peso.

### 2. REQUERIMIENTOS DE FRONTEND (UI/UX)
- **Framework:** SvelteKit (o Svelte puro) con TailwindCSS para mantener el bundle final lo más ligero y responsivo posible.
- **Visualización de PDF:** Integra `PDF.js` de Mozilla para renderizar las páginas del PDF en elementos `<canvas>` de HTML5, permitiendo al usuario reordenar páginas visualmente (Drag and Drop), seleccionar rangos para dividir y previsualizar dónde se aplicará el OCR.

### 3. TAREAS INICIALES REQUERIDAS
Por favor, genera para mí:
1. La **estructura completa de carpetas y archivos** recomendada para este proyecto en Tauri v2, mostrando claramente dónde interactúan los comandos de Rust con las llamadas nativas de Swift/Kotlin.
2. El código de un comando de Rust (`tauri::command`) que use `lopdf` y `rayon` para **unir una lista de rutas de archivos PDF en un único archivo final de forma eficiente**.
3. La abstracción en Rust (un trait o estructura condicional con `#[cfg(target_os = "...")]`) que prepare el terreno para conectar el OCR nativo de Windows, macOS, iOS y Android.
