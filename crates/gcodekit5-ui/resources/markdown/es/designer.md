# Diseñador
  <img src="../../help_images/designer.png" alt="Diseñador" width="700">

Diseñador sirve para crear y editar e importar diseños vectoriales así como la importación de Imágenes para el grabado

## Acciones principales
- Dibujar primitivas (rectángulos, círculos, líneas, elipses, polilineas, triangulos, Polígonos, Piñones, Engranajes)
- Añadir texto
- Añadir imágenes incluso hacer una composición de varias junto con los objetos vectoriales
- Definir independientemente los parámetros de grabado de cada imagen mediante sus propiedades
- Definir parámetros globales para aquellos objetos vectoriales que no precisen parámetros independientes
- Definir independientemente los parámetros de cada objeto vectoriales mediante sus propiedades
- Reordenar los Objetos en el panel de Objetos que se utilizará para la generación del G-code
- Importar archivos DXF y SVG.
- Advertencia de Fuera de Límites. Si algún punto sale fuera del área de trabajo, aparecerá en el G-code una advertencia de Fuera de Límites para que el usuario determine que hacer
- Exportar a G-code o SVG
- Generar el "Frame" para ajuste del material en máquina
- Generar el G-code final. Al generar el G-code se salta a la pestaña del Visualizador para comprobar el resultado antes de lanzar el trabajo.

---
## Propiedades globales
  <img src="../../help_images/global_properties.png" alt="Propiedades Globales" width="600">

Pulsando el botón "Configuración de Herramienta" se abre la ventana para la configuración Global de trabajo. Esta Configuración se utilizará para todos los objetos vectoriales que tengan marcado el CheckBox "Usar Valores Globales" en las propiedades de objeto "Configuración Láser (Objeto)"

---
## Panel de Propiedades individuales de objeto
  <img src="../../help_images/individual_properties.png" width="300">
      En el panel lateral derecho, cuando se selecciona un objeto, aparecen sus propiedades:
        <li> Posición</li>
        <li> Tamaño</li>
        <li> Rotación</li>
        <li> Esquinas (redondeo)</li>
        <li> Operaciones Geométricas (desfase, empalme, chaflán)</li>
        <li> Propiedades CAM</li>
        <li> Configuración individual Láser (velocidad, potencia y pasadas) de un objeto determinado</li>
---
## Panel de Objetos
  <img src="../../help_images/order_objects_1.png" width="300">
      En el panel de objetos aparece la lista de objetos con:
        <li> Número de orden</li>
        <li> Tipo de objeto (Rectángulo, Circulo, Path, etc.) e identificación #</li>
        <li> Nombre del Objeto</li>
        <li> El número de orden es editable y sirve para organizar los objetos a la hora de generar el G-code para que los objetos se ejecuten es ese orden.</li>
        <li> El Nombre también es editable, de modo que se puedan identificar los objetos convenientemente</li>

  <img src="../../help_images/order_objects_2" width="300">
  <li> </li>

---
## Generador de Gcode y Frame
  <img src="../../help_images/gcode_frame.png" alt="Generador de Gcode y Frame" width="200">
        - En el panel de la izquierda de Diseñador están los botones de "Generar G-Code" y "Frame". Después de terminar el diseño, es conveniente generar el perímetro del trabajo para poder enviar a la máquina y centrar el material convenientemente. Una vez realizado este proceso, volver al diseñador para generar el G-Code mediante el botón. Una vez Generado, automáticamente saltará a la pestaña de Visualizador para comprobar como se realizará el trabajo. Una vez conforme, ir al Control de Máquina para lanzar el trabajo.
---

## Relacionado
[Visualizador](help:visualizer)
[Control de Máquina](help:machine_control)
[Índice](help:index)

