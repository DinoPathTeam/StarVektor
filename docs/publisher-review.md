# Lectura de acuerdos de editores

Registro de lo que dicen los acuerdos de licencia (EULA) de los editores cuyos juegos podrían usarse para **evaluar** StarVektor. Es una lectura del equipo, no asesoría legal. Los textos pueden cambiar y varían por región y edición: antes de depender de uno, se relee el vigente.

> **Etapa temprana.** Este registro crece a medida que se revisan editores. Lo que no aparece aquí no ha sido revisado.

**Última revisión:** 2026-10-27.

## Cómo se interpreta

- Ningún acuerdo leído menciona la captura ni la generación de fotogramas. Solo el de Epic menciona la IA (entrenamiento de IA generativa, ver abajo). El silencio no permite ni prohíbe de forma explícita.
- Lo que cada acuerdo regula son **actos**, y la intención no cuenta. Por eso la defensa del proyecto es describir con precisión lo que hace y lo que no, ver [usage-statement.md](usage-statement.md).
- Estados: **Sin cláusula en contra** (según la lectura), **Cláusulas amplias** (se evalúa con cautela y, idealmente, permiso escrito), **Excluido por ahora** (no se usa), **Sin verificar**.

## Plataformas

| Plataforma | Cláusulas relevantes | Estado |
|---|---|---|
| Steam (Acuerdo de suscriptor, §2.G) | Prohíbe copiar, reproducir, modificar, desensamblar, descompilar, aplicar ingeniería inversa, crear obras derivadas y quitar avisos de propiedad de los contenidos y servicios, salvo permiso o ley aplicable. Uso personal y sin fines comerciales. | Sin cláusula en contra para observación de imagen. §4.B (software de terceros no autorizado) aún por leer completa. |
| Epic Games Store (EULA y términos de los productos con licencia) | Ingeniería inversa, modificar, descompilar o crear obras derivadas; programas no autorizados para ganar ventaja en modos online; usar los productos fuera de lo expresamente permitido. **Prohíbe usar código o contenido extraído de los productos como datos de entrenamiento de un programa de IA generativa** (o como entrada con la que se entrene). Define el término de forma amplia, y exceptúa a los programas que operan solo sobre el contenido original (por ejemplo, ampliación de imágenes). Según lo observado, el mismo texto aparece en los juegos de Epic revisados. | Sin cláusula en contra mientras no se entrene con juegos de terceros (D4 y regla 7 de la [política de datos](data-policy.md)). No se cuenta con la excepción de "solo contenido original": una generación de fotogramas crea contenido visual nuevo. |

## Editores

| Editor / juego | Cláusulas relevantes | Estado |
|---|---|---|
| Bethesda (Fallout 4 y similares) | Prohíbe uso comercial, distribución, copias, uso en red/online, ingeniería inversa y modificar el software. Nada sobre software de terceros, minería de datos ni grabación. | Sin cláusula en contra |
| Square Enix (FFVII Remake) | §3: uso comercial, copiar o almacenar el producto, ingeniería inversa, eludir protecciones, y crear programas que imiten datos o funcionalidad del producto. | Sin cláusula en contra, bajo la declaración (no guardar fotogramas; imitamos una función de la pantalla, no del juego) |
| 505 Games (Death Stranding) | Prohíbe extraer información y aplicar ingeniería inversa. Nada sobre overlays ni IA. | Sin cláusula en contra |
| Ubisoft | Prohíbe trainers, macros y programas de trampa; el juego puede vigilar la memoria en busca de programas de terceros que interactúen. | Solo juegos sin conexión |
| CD PROJEKT RED (Cyberpunk 2077, The Witcher 3) | Prohíbe ingeniería inversa y "cheats", definidos como software que interactúe con o afecte el juego "de cualquier forma", incluidos programas que recojan información leyendo memoria. También usar software similar junto con el suyo y extraer el juego para minería de texto y datos. | Cláusulas amplias: evaluar solo bajo la declaración y, a futuro, solicitar permiso |
| Pearl Abyss (Crimson Desert) | Art. 5 del EULA de Black Desert: ingeniería inversa, programas no autorizados, interferir con medidas técnicas, minería de datos y crear programas que imiten datos o funcionalidad del juego. | Cláusulas amplias. **El EULA de Crimson Desert está sin verificar**: confirmar que el texto leído sea el suyo |
| Warner Bros. Games | §2.1(g): prohíbe software no autorizado "diseñado para interactuar con el software de cualquier forma", incluso para recoger información. | Excluido por ahora, hasta contar con argumentos y permiso |
| Sony (PlayStation PC) | Texto redactado para consolas; aplicación a PC poco clara. | Excluido por decisión del proyecto |
| Remedy (EULA en Steam, id 870780) | Licenciante: Remedy Entertainment Plc; el texto no nombra el juego. §2.3(iv): no "recolectar información" del software o su contenido. §2.3(v): no desensamblar, descompilar, descifrar, hackear, emular, explotar, aplicar ingeniería inversa ni modificar. §2.3(vi): no eludir medidas técnicas de protección. Uso personal y sin fines comerciales. Nada sobre overlays, grabación ni IA. Su sitio web no tiene un EULA aparte: la política de privacidad y los términos y condiciones cubren solo los servicios y el sitio. | Sin cláusula en contra, bajo la declaración. Punto a vigilar: §2.3(iv); no recolectamos información del juego, medimos nuestra propia salida. Confirmar en Steam a qué juego corresponde el id. |
| SCS Software (Euro Truck Simulator 2) | No se localizó un EULA propio en su sitio, su blog ni su página de proyecto. En los archivos de instalación solo hay avisos de licencia de bibliotecas de terceros (Bullet, DirectStorage, HIDAPI, Dear ImGui, MoltenVK, OpenXR, NVAPI, PhysX, wxWidgets, Xiph.Org). | Sin EULA propio localizado: aplicaría el acuerdo de Steam. No es lo mismo que "no existe"; pendiente de confirmar con el editor |
| Resto de editores | No revisados. | Sin verificar |

## Anticheat

El riesgo principal en juegos online es operativo más que contractual: un anticheat puede tratar una capa inyectada como intrusión. La documentación de Valve advierte que VAC puede dar falsos positivos con overlays y software de grabación. Por eso el alcance es de un solo jugador y sin conexión (D9). El estado de compatibilidad de cada anticheat en Linux cambia, y se consulta por título antes de probar.

## Próximos pasos

- Leer §4.B del acuerdo de Steam completa.
- Confirmar el EULA de Crimson Desert y localizar el de Control.
- Revisar los editores restantes que entren en el conjunto de evaluación.
- Con los resultados de la Fase 1, solicitar permiso de comparación numérica a los editores con cláusulas amplias, u ofrecer el software para su uso en desarrollo interno.
