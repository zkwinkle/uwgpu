---
toc: yes
header-includes: yes
geometry: "left=1.6cm,right=1.6cm,top=1.6cm,bottom=1.6cm"
gradient: no
underline: no
---

# 1. Introducción

Este Documento de Requisitos de Software (SRS) detalla el desarrollo de una
suite de microbenchmarks diseñada para medir características de rendimiento de
diferentes GPUs a través de diferentes plataformas y APIs gráficos. El software
constará de dos interfaces principales: una web, para ejecutar los
microbenchmarks usando WebGPU desde un navegador, y una CLI, para ejecutar los
microbenchmarks de forma nativa sobre Vulkan. Este documento especifica los
requisitos funcionales, las características del usuario, el alcance del
producto, y las verificaciones.

## 1.1. Propósito

El propósito principal del software es proporcionar una suite de
microbenchmarks para medir características específicas del rendimiento de GPUs
en plataformas que soportan WebGPU y Vulkan.

También facilita la ejecución de microbenchmarks propios de los usuarios.

Este software aborda la necesidad de obtener datos de rendimiento detallados y
comparativos a través de diferentes combinaciones de hardware y plataformas,
permitiendo a desarrolladores e investigadores optimizar su software para
diferentes escenarios y evaluar el desempeño general de las GPUs.

## 1.2. Alcance

El alcance del software, denominado "wgpu microbench" abarca el diseño,
desarrollo e implementación de los microbenchmarks para evaluar características
del hardware de GPUs y las interfaces para ejecutarlos y visualizar resultados.

El software tiene 2 objetivos principales:

- Simplificar el proceso de escribir microbenchmarks para GPU.
- Proveer un banco de microbenchmarks comparativos con operaciones comunes para
  entender las capacidades del hardware sobre el que se ejecutan.

El software incluirá dos interfaces que dan acceso a estas capacidades:

1. Interfaz web: Permite ejecutar microbenchmarks desde un navegador,
   recolectar datos, visualizar resultados, y proporcionar una opción para
   ejecutar microbenchmarks personalizados.
2. Interfaz CLI: Permite ejecutar microbenchmarks nativamente, compilando a
   Vulkan y ofrece resultados en texto.

 El producto es autónomo y no interactúa con sistemas externos, aunque opera
 dentro del contexto los APIs gráficos mencionados y las plataformas sobre las
 que se ejecutan.

## 1.3. Resumen del producto

### 1.3.1. Perspectiva del producto

Los microbenchmarks se implementarán utilizando el API de gráficos _wgpu_ que
se puede correr en navegadores web que tengan soporte para WebGPU o nativamente
traduciendo a los APIs nativos Vulkan, Metal y DX12. Aunque solo se verificará
la capacidad de correr los microbenchmarks sobre Vulkan, por las razones
especificadas en la sección de limitaciones.

La figura [TODO: poner # figúra] muestra las plataformas en las que los
microbenchmarks pueden ser evaluados y las capas de APIs gráficos, plataformas
y hardware.

[TODO: Figura de capas y plataformas compatibles]

No forma parte de un sistema mayor, el software es el producto entero que se
ofrece. El desarrollo abarca todas las partes del producto, incluyendo la
implementación de interfaces web y CLI.

### 1.3.2. Funciones del producto

Las principales funciones del software incluyen:

- **Ejecución de banco de microbenchmarks:** Medición de características
  específicas del rendimiento de GPUs.
- **Recolección y visualización de datos:** En la interfaz web, se recolectan y
  visualizan resultados; en la interfaz CLI, se generan resultados en texto.
- **Ejecución de microbenchmarks personalizados:** Facilita a los usuarios
  ejecutar y evaluar microbenchmarks personalizados a través de ambas
  interfaces.

[TODO: Diagrama mostrando funciones dependiendo de interfaz?]

### 1.3.3. Características del usuario

Los usuarios esperados incluyen desarrolladores de software de gráficos por
computadora e investigadores en GPU compute u otros temas de rendimiento en
GPU. Se espera que los usuarios tengan conocimientos sobre WebGPU y APIs
gráficas, así como habilidades para interpretar resultados de benchmarks y
optimizar el rendimiento de software para GPUs.

### 1.3.4. Limitaciones

El API de gráficos sobre el que se implementará el software, _wgpu_, permitiría
compilar el software nativamente (interfaz CLI) para cualquier plataforma con
los APIs gráficos Vulkan, Metal o DX12; lo cual incluye los sistemas operativos
Windows, Mac y Linux. Sin embargo, el desarrollador se ve limitado en las
plataformas a las que tiene acceso ya que no cuenta con hardware de Apple
(necesario para utilizar la API gráfica Metal) ni licencia de Windows
(necesario para utilizar la API gráfica DX12). Por lo tanto, solo se podrán
verificar el funcionamiento del software para la plataforma Linux y
consecuentemente el API de gráficos Vulkan.

De igual manera, la interfaz web solo se podrá verificar para los navegadores web disponibles en Linux (notablemente excluyendo a Safari que tiene soporte experimental de WebGPU) y en Android (notablemente excluyendo verificación en iOS).

## 1.4. Definiciones

- **Microbenchmark:** Prueba que mide el rendimiento de alguna característica
  específica de bajo nivel. Se diferencia de un "benchmark" ya que ese es un
  concepto más general que aplica a cualquier tipo de prueba que podría incluir
  el rendimiento de sistemas enteros.
- **GPU Compute:** Uso del GPU para cálculos computacionales de manera general.

## 1.5. Siglas y abreviaturas

**GPU:** Unidad de procesamiento gráfico. **CLI:** Interfaz de línea de
comandos.

# 2. Referencias

- [Proyecto wgpu](https://github.com/gfx-rs/wgpu)
- [Estándar de WebGPU](https://gpuweb.github.io/gpuweb/)
- [Proyecto µVkCompute](https://github.com/google/uVkCompute)
- [Dissecting GPU Memory Hierarchy through
  Microbenchmarking](https://arxiv.org/abs/1509.02308)
- [Demystifying GPU Microarchitecture through
  Microbenchmarking](https://courses.cs.washington.edu/courses/cse470/24sp/readings/Demystifying_GPU_microarchitecture_through_microbenchmarking.pdf)
- [Nvidia CUDA Programming
  Guide](https://developer.download.nvidia.com/compute/cuda/1.0/NVIDIA_CUDA_Programming_Guide_1.0.pdf)
- [GPU Atomic Performance Modeling with Microbenchmarks](https://vulkan.org/user/pages/09.events/vulkanised-2024/vulkanised-2024-devon-mckee.pdf)

# 3. Requisitos

## Microbenchmarks

## 3.1. Funciones

Las funciones se especificaran en subsecciones para cada una de las interfaces
propuestas y otra subsección sobre los requerimientos de los microbenchmarks
que ambas interfaces implementará.

### 3.1.1. Interfaz web

- \[**11001**\]: Cuando el usuario inicie la acción de ejecutar microbenchmarks, la interfaz web deberá ejecutar el conjunto de microbenchmarks predeterminados.
- \[**11002**\]: Si el usuario indica que solo se ejecute uno de los microbenchmarks (puede ser a través de un selector), cuando el usuario inicie la acción de ejecutar microbenchmarks, la interfaz web deberá ejecutar solo el microbenchmark predeterminado que fue indicado.
- \[**11003**\]: La interfaz web deberá proporcionar un botón de "desactivar recopilación de datos" para el usuario.
- \[**11004**\]: Si se ejecuta el conjunto completo o solo uno de microbenchmarks predeterminados mientras el botón de "desactivar recopilación de datos" está inactivo, la interfaz web deberá recopilar los datos de rendimiento obtenidos de los microbenchmarks, el hardware y la plataforma del usuario y guardar esos datos en una base de datos.
- \[**11005**\]: Si se ejecuta el conjunto de microbenchmarks predeterminados mientras el botón de "desactivar recopilación de datos" está activo, la interfaz web no deberá recopilar ningún dato.
- \[**11006**\]: Cuando se terminan de ejecutar el conjunto completo o solo uno de los microbenchmarks predeterminados, la interfaz web deberá mostrar en la interfaz la plataforma, navegador web y hardware identificados para el usuario así como las características de rendimiento obtenidas de los microbenchmarks.
- \[**11007**\]: La interfaz web deberá proporcionar una caja de texto para que el usuario escriba el código de su propio microbenchmark personalizado.
- \[**11008**\]: Cuando el usuario lo indique, la interfaz web deberá ejecutar el microbenchmark personalizado del usuario.
- \[**11009**\]: Si el código del microbenchmark personalizado no puede compilar o ejecutarse por algún otro motivo cuando el usuario intenta ejecutarlo, entonces la interfaz web deberá indicar el error al usuario con una _alerta_.
- \[**11010**\]: Cuando finalice la ejecución correcta del microbenchmark personalizado, la interfaz web deberá mostrar cuánto tiempo duró en ejecutarse.
- \[**11011**\]: Si la plataforma o hardware del usuario no tiene soporte para alguna operación que se utilice en alguno de los microbenchmarks predeterminados cuando el usuario intente ejecutarlo, la interfaz web deberá indicar esta falta de soporte de operación al usuario y no ejecutará el microbenchmark específico.
- \[**11012**\]: La interfaz web deberá permitir a los usuarios accesar y ver estadísticas de los datos recopilados.
- \[**11013**\]: Cuando el usuario acceda las estadísticas, la interfaz web deberá obtener las estadísticas del servidor encargado de la base de datos.
- \[**11014**\]: Si el usuario aplica filtros (posiblemente a través de selectores) cuando accede las estadísticas, la interfaz web deberá obtener las estadísticas para un subconjunto de datos filtrados a como lo indicó el usuario; los filtros disponibles son para un hardware específico, un navegador web específico, una plataforma específica y un microbenchmark específico.
- \[**11015**\]: Si la petición falla (por algún error de conectividad o cualquier otra razón) cuando se intentan descargar las estadísticas para visualización, la interfaz web deberá abortar la operación de visualización e indicar el fallo al usuario por medio de una _alerta_, indicándole que vuelva a intentar más tarde.
- \[**11016**\]: La interfaz web deberá proveer un botón para descargar todos los datos recopilados de ejecuciones de microbenchmarks en formato CSV.
- \[**11017**\]: Si la petición falla (por algún error de conectividad o cualquier otra razón) cuando se intentan descargar el archivo de formato CSV con todos los datos de microbenchmarks realizados, la interfaz web deberá abortar la operación e indicar el fallo al usuario por medio de una _alerta_, indicándole que vuelva a intentar más tarde.

### 3.1.2. Interfaz CLI

- \[**12001**\]: Cuando el usuario ejecute el comando de la interfaz CLI, esta deberá ejecutar el conjunto de microbenchmarks predeterminados.
- \[**12002**\]: Si el usuario indica que solo se ejecute uno de los microbenchmarks (puede ser a través de una bandera en el comando), cuando el usuario ejecute el comando, la interfaz CLI deberá ejecutar solo el microbenchmark predeterminado que fue indicado.
- \[**12003**\]: Cuando se terminan de ejecutar el conjunto completo o solo uno de los microbenchmarks predeterminados, la interfaz CLI deberá mostrar dentro de la terminal las características de rendimiento obtenidas de los microbenchmarks.
- \[**12004**\]: La interfaz CLI deberá proporcionar la opción (posiblemente a través de una bandera en el comando) de leer un archivo de texto donde el usuario haya escrito el código de su propio microbenchmark personalizado y que este sea ejecutado.
- \[**12005**\]: Si el usuario la india al comando que lea un archivo de texto para ejecutar su propio microbenchmark personalizado, al ejecutar el comando la interfaz CLI no deberá ejecutar ninguno de los microbenchmarks predeterminados.
- \[**12006**\]: Si el código del microbenchmark personalizado no puede compilar o ejecutarse por algún otro motivo cuando el usuario intenta ejecutarlo, entonces la interfaz CLI deberá indicar el error al usuario con un mensaje de texto en la terminal.
- \[**12007**\]: Cuando finalice la ejecución correcta del microbenchmark personalizado, la interfaz CLI deberá mostrar cuánto tiempo duró en ejecutarse.
- \[**12008**\]: Si el hardware del usuario no tiene soporte para alguna operación que se utilice en alguno de los microbenchmarks predeterminados cuando el usuario intente ejecutarlo, la interfaz CLI deberá indicar esta falta de soporte de operación al usuario y no ejecutará el microbenchmark específico.

### 3.1.3. Microbenchmarks

- \[**13001**\]: El banco de microbenchmarks deberá incluir al menos 2 microbenchmarks con métodos distintos de realizar convoluciones.
- \[**13002**\]: El banco de microbenchmarks deberá incluir al menos 2 microbenchmarks con métodos distintos de realizar multiplicación matricial.
- \[**13003**\]: El banco de microbenchmarks deberá incluir al menos 2 microbenchmarks con métodos distintos de realizar reducciones.
- \[**13004**\]: El banco de microbenchmarks deberá incluir al menos 2 microbenchmarks con métodos distintos de realizar la operación conocida como suma de prefijos o "scan".
- \[**13005**\]: El banco de microbenchmarks deberá incluir al menos un microbenchmarks para medir el ancho de banda de accesos de memoria secuenciales del GPU.
- \[**13006**\]: El banco de microbenchmarks deberá incluir al menos un microbenchmarks para medir el ancho de banda de accesos de memoria desordenados del GPU.
- \[**13007**\]: El banco de microbenchmarks deberá incluir al menos un microbenchmarks para medir el ancho de banda de copiar memoria entre buffers del GPU.
- \[**13008**\]: El banco de microbenchmarks deberá incluir al menos un microbenchmarks para medir el ancho de banda de copiar memoria de buffer a texturas del GPU.
- \[**13000**\]: El banco de microbenchmarks deberá incluir al menos un microbenchmarks para medir el ancho de banda de copiar memoria entre texturas del GPU.

## 3.2. Requisitos de usabilidad

- \[**20001**\]: Cada microbenchmark individual deberá producir resultados consistentes con una variación de no más del 5% en múltiples ejecuciones en el mismo hardware y plataforma en un computador que no esté realizando otros procesamientos significativos.
- \[**20002**\]: Cada microbenchmark individual deberá ejecutarse en menos de 5 minutos en la computadora del desarrollador, específicamente una laptop equipada con una GPU integrada de un procesador Intel(R) Core(TM) i7-1260P de 12ª generación.

**Nota:** Este límite de tiempo ha sido seleccionado de manera arbitraria, pero
razonable, basado en lo que se considera aceptable para la mayoría de los
usuarios. Se anticipa que si un microbenchmark se completa dentro de este
periodo en el hardware del desarrollador, probablemente se ejecutará más
rápido en la mayoría de sistemas modernos en especial si cuentan con un GPU
discreto. En casos donde la ejecución exceda este límite en otros
dispositivos, se espera que el tiempo adicional no impacte significativamente
la usabilidad.

- \[**20003**\]: La interfaz web deberá ser portable, funcionando de manera efectiva en todas las plataformas objetivo oficiales: Chromium y Firefox, en Linux y en Android; garantizando un rendimiento y usabilidad consistentes en estos entornos de manera individual (es decir, que el rendimiento no tiene que ser consistente entre plataformas diferentes).

## 3.3. Interfaces externas

No aplica.

## 3.4. Requisitos de rendimiento

No aplica.

## 3.5. Requisitos de la base de datos lógica

### 3.5.1. Tipos de Información Utilizada por Diversas Funciones

- \[**51001**\]: La base de datos deberá almacenar los resultados de los microbenchmarks, incluyendo métricas de rendimiento (por ejemplo, tiempo de ejecución o uso de memoria, variará dependiendo del microbenchmark), información de la plataforma (sistema operativo, navegador web, versión del navegador) y detalles del hardware (módelo del GPU).

### 3.5.2. Frecuencia de Uso

No aplica.

### 3.5.3. Capacidades de Acceso

- \[**53001**\]: La base de datos deberá permitir filtrar los datos de microbenchmarks según diversos criterios como plataforma, configuración de hardware y métricas de rendimiento específicas.
- \[**53002**\]: Si los usuarios solicitan descargar los datos, la base de datos deberá proporcionar capacidades para generar y descargar archivos CSV que contengan todo el cuerpo de datos.

### 3.5.4. Entidades de Datos y Sus Relaciones

- \[**54001**\]: La base de datos deberá incluir una entidad para cada categoría de microbenchmark (como convolución, reducción, otros) con los datos relevantes para esa categoría.
- \[**54002**\]: La base de datos deberá tener la capacidad de incluir una entidad para microbenchmark específico si requiere datos adicionales específicos al microbenchmark.
- \[**54003**\]: La base de datos deberá incluir una entidad para plataformas que incluya la información de sistema operativo, navegador web, versión de navegador y hardware.

### 3.5.5. Restricciones de Integridad

- \[**55001**\]: Siempre que se almacene un resultado de microbenchmark, la base de datos deberá imponer integridad referencial, asegurando que un resultado de microbenchmark no pueda existir sin datos válidos asociados de plataforma y hardware.

### 3.5.6. Seguridad

No aplica.

### 3.5.7. Requerimientos de retención de datos

No aplica.

## 3.6. Restricciones de diseño

## 3.7. Atributos del sistema de software

## 3.8. Información de soporte

# 4. Verificación

# 5. Apéndices

## 5.1. Supuestos y dependencias

## 5.2. Matriz de trazabilidad
