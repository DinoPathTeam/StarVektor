# Lectura de acuerdos de editores

Registro de lo que dicen los acuerdos de licencia (EULA) de los editores cuyos juegos podrían usarse para **evaluar** StarVektor. Es una lectura del equipo, no asesoría legal. Los textos pueden cambiar y varían por región y edición: antes de depender de uno, se relee el vigente.

**Última revisión:** 2026-10-26.

## Cómo se interpreta

- Ningún acuerdo leído menciona la captura ni la generación de fotogramas, ni IA. Eso es silencio: no los permite ni los prohíbe de forma explícita.
- Lo que cada acuerdo regula son **actos**, y la intención no cuenta. Por eso la defensa del proyecto es describir con precisión lo que hace y lo que no, ver [usage-statement.md](usage-statement.md).
- Estados: **Sin cláusula en contra** (según la lectura), **Cláusulas amplias** (se evalúa con cautela y, idealmente, permiso escrito), **Excluido por ahora** (no se usa), **Sin verificar**.

## Plataformas

| Plataforma | Cláusulas relevantes | Estado |
|---|---|---|
| Steam (Acuerdo de suscriptor, §2.G) | Prohíbe copiar, reproducir, modificar, desensamblar, descompilar, aplicar ingeniería inversa, crear obras derivadas y quitar avisos de propiedad de los contenidos y servicios, salvo permiso o ley aplicable. Uso personal y sin fines comerciales. | Sin cláusula en contra para observación de imagen. §4.B (software de terceros no autorizado) aún por leer completa. |
| Epic Games Store (EULA) | Ingeniería inversa; programas no autorizados para ganar ventaja en modos online. | Sin cláusula en contra. Se tomó como equivalente a Steam. |

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
| Remedy / 505 (Control) | No se localizó el EULA vigente. | Sin verificar |
| Resto de editores | No revisados. | Sin verificar |

## Anticheat

El riesgo principal en juegos online es operativo más que contractual: un anticheat puede tratar una capa inyectada como intrusión. La documentación de Valve advierte que VAC puede dar falsos positivos con overlays y software de grabación. Por eso el alcance es de un solo jugador y sin conexión (D9). El estado de compatibilidad de cada anticheat en Linux cambia, y se consulta por título antes de probar.

## Próximos pasos

- Leer §4.B del acuerdo de Steam completa.
- Confirmar el EULA de Crimson Desert y localizar el de Control.
- Revisar los editores restantes que entren en el conjunto de evaluación.
- Con los resultados de la Fase 1, solicitar permiso de comparación numérica a los editores con cláusulas amplias, u ofrecer el software para su uso en desarrollo interno.
