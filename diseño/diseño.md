---
toc: yes
header-includes: yes
geometry: "left=1.6cm,right=1.6cm,top=1.6cm,bottom=1.6cm"
gradient: no
underline: no
colorlinks: yes
titlepage: yes

title: Documento de Diseño
subtitle: µwgpu
author: Ignacio Vargas Campos
id: 2019053776
date: II Semestre 2024
institute: Instituto Tecnológico de Costa Rica
department: Escuela de Ingeniería en Computadores
---

# 1. Detalles del Documento

## 1.1. Fecha de la Versión y Estatus

- Fecha: 6 de septiembre de 2024
- Estatus: Versión 1.0

## 1.2. Organización

El proyecto será llevado a cabo como parte de un proyecto de investigación en el Instituto Tecnológico de Costa Rica.

## 1.3. Autor

Ignacio Vargas Campos

## 1.4. Historia de Cambios

| Versión | Fecha                  | Descripción de Cambios                 |
|---------|------------------------|----------------------------------------|
| 1.0     | 6 de septiembre de 2024| Versión inicial del documento          |


# 2. Introducción

## 2.1. Propósito

El objetivo principal de este software es proporcionar un conjunto de
microbenchmarks que permitan medir características específicas del rendimiento
de GPUs en plataformas compatibles con WebGPU y Vulkan.

Además, el software facilita a desarrolladores la creación y ejecución de sus
propios microbenchmarks personalizados a través de una biblioteca.

Esta herramienta responde a la necesidad de obtener datos de rendimiento
detallados y comparativos a través de diversas combinaciones de hardware y
plataformas, permitiendo a desarrolladores e investigadores optimizar sus
aplicaciones para diferentes escenarios y evaluar el rendimiento general de las
GPUs.

## 2.2. Alcance

El software, denominado "µwgpu," abarca el diseño, desarrollo e implementación
de microbenchmarks para evaluar las características de hardware de GPUs, así
como las interfaces necesarias para su ejecución y visualización de resultados.

El proyecto tiene dos objetivos centrales:

- Simplificar la creación de microbenchmarks para GPU de manera
multi-plataforma.
- Proveer un banco de microbenchmarks comparativos que incluyan operaciones
comunes para evaluar las capacidades del hardware en ejecución.

El software ofrecerá dos tipos de interfaces de usuario:

1. **Interfaz web:** Permite ejecutar microbenchmarks directamente desde un
navegador, recolectar datos y mostrar resultados.
2. **Interfaz de línea de comandos (CLI):** Permite ejecutar microbenchmarks de
manera nativa, compilando para Vulkan y proporcionando los resultados en
formato de texto.

## 2.3. Contexto

El software "µwgpu" se desarrollará como parte de un proyecto de investigación del Instituto Tecnológico de Costa Rica, orientado a abordar las necesidades de la comunidad de desarrollo de software gráfico, tanto en el ámbito académico como en la industria. Este proyecto surge del interés de proporcionar herramientas útiles para el análisis y optimización del rendimiento de GPUs, especialmente en entornos que utilizan tecnologías emergentes como WebGPU y Vulkan.

El producto será utilizado principalmente por desarrolladores e investigadores que buscan recolectar y analizar datos de rendimiento detallados a través de diferentes configuraciones de hardware y software. Su objetivo es facilitar el estudio de las capacidades de las GPUs, así como la creación de programas y algoritmos que puedan ejecutarse de manera eficiente en una amplia variedad de dispositivos y plataformas. Además, esta herramienta también resultará valiosa para empresas tecnológicas que buscan optimizar sus productos y para fabricantes de hardware que desean evaluar el rendimiento de sus dispositivos en diferentes escenarios.

## 2.4. Resumen

- Resumen general del contenido del documento, detallando el diseño.

## 2.5. Interesados

- **Desarrolladores e Ingenieros en GPU**: Utilizan este proyecto para
optimizar el rendimiento de aplicaciones mediante microbenchmarks específicos
de WebGPU. Buscan herramientas precisas para identificar puntos críticos de
rendimiento y mejorar la eficiencia.
- **Investigadores Académicos**: Emplean los microbenchmarks en estudios y
experimentos relacionados con el rendimiento de GPUs, buscando datos exactos
que puedan validar sus teorías y apoyar la publicación de resultados.
- **Empresas de Tecnología y Desarrollo de Software**: Implementan los
microbenchmarks para optimizar el rendimiento y la experiencia de usuario en
sus productos, identificando y solucionando problemas antes del lanzamiento.
- **Proveedores de Hardware**: Evalúan el rendimiento de sus GPUs en diversos
escenarios para ajustar y mejorar sus productos, basándose en los resultados
proporcionados por los microbenchmarks.
- **Usuarios Finales (Desarrolladores y Usuarios de Aplicaciones)**: Se
benefician indirectamente de las mejoras en el rendimiento de las aplicaciones,
lo cual afecta positivamente la calidad de su experiencia.

# 3. Referencias

- [Proyecto wgpu](https://github.com/gfx-rs/wgpu)
- [Estándar de WebGPU](https://gpuweb.github.io/gpuweb/)
- [Proyecto µVkCompute](https://github.com/google/uVkCompute)
- [Dissecting GPU Memory Hierarchy through
Microbenchmarking](https://arxiv.org/abs/1509.02308)
- [Demystifying GPU Microarchitecture through
Microbenchmarking](https://courses.cs.washington.edu/courses/cse470/24sp/readings/Demystifying_GPU_microarchitecture_through_microbenchmarking.pdf)
- [Nvidia CUDA Programming
Guide](https://developer.download.nvidia.com/compute/cuda/1.0/NVIDIA_CUDA_Programming_Guide_1.0.pdf)
- [GPU Atomic Performance Modeling with
Microbenchmarks](https://vulkan.org/user/pages/09.events/vulkanised-2024/vulkanised-2024-devon-mckee.pdf)

# 4. Glosario

- **Microbenchmark**: Prueba destinada a medir el rendimiento de una
característica específica de bajo nivel. A diferencia de un "benchmark"
general, un microbenchmark se centra en pruebas específicas, generalmente a
nivel de componentes.
- **GPU Compute**: Uso de la GPU para realizar cálculos computacionales de
propósito general.
- **GPU**: Unidad de Procesamiento Gráfico. **CLI**: Interfaz de Línea de
Comandos.

# 5. Perspectivas de diseño

- Design concers de la perspectiva

- Design elements, defined by that viewpoint, specifically the types of design
entities, attributes, relationships, and constraints introduced by that
viewpoint or used by that viewpoint (which may have been defined elsewhere).
These elements may be realized by one or more design languages; Formal or
informal consistency and completeness tests to be applied to the view;

- Evaluation or analysis techniques to be applied to a view; and

- Heuristics, patterns, or other guidelines to assist in construction or
synthesis of a view.

- An SDD shall include a rationale for the selection of each selected
viewpoint.

- Each design entity shall have a name, a type, and purpose.

## 5.1. Contexto

El diagrama de la figura 1 muestra los casos de uso que provee el sistema en el
lenguaje de diagrama de casos de uso UML. Muestra los servicios proveídos por el
software y su usuario principal.

![Casos de uso de µwgpu.](./diagramas/UML Casos de Uso.svg){width=50%}

El diagrama es bastante simple, dentro del "sistema" de µwgpu se tienen las
funciones que este paquete de software provee a un desarrollador o
investigador.

El sistema no requiere de más actores que del usuario directo que utilice el
software, el cual la mayoría del tiempo será un desarrollador.

También se muestra como un servidor será el encargado de recolectar los datos
de rendimiento históricos y proporcionarlos a usuarios.

## 5.2. Composición

![Diagrama de composición de µwgpu.](./diagramas/Composition Diagram.svg){width=70%}

El diagrama de la figura 2 muestra, utilizando el lenguaje de diagrama de componentes UML, la composición de los subsistemas de µwgpu:

- La biblioteca principal "uwgpu" que se utilizar para crear microbenchmarks.
Sus dependencias que cada microbenchmark debe proveer de acuerdo a lo que
realice y recursos que utilice.
- La biblioteca de "microbenchmarks". Nótese que este bloque solo se muestra 1
componente llamado "microbenchmark". En realidad, van a haber muchos
microbenchmarks con nombres más específicos, pero sus interfaces todas van a
tener la misma forma: todos proveen los mismos recursos a uwgpu y las mismas
interfaces a las interfaces de usuario. Por lo tanto se simplificó el bloque a
mostrar solo 1 "microbenchmark" de ejemplo.
- Las interfaces de usuario que interactúan con la biblioteca proveedora de
microbenchmarks para ejecutarlos y obtener sus resultados.


## 5.3. Lógica

Clases, interfaces, relaciones.

- Benchmark
- BenchmarkComputePipeline
- GPUContext
- BenchmarkResults

## 5.4. Dependencia

No aplica.

## 5.5. Información (Datos persistentes)

Por definir. Se contará con un servidor que guarda información histórica de rendimiento para las ejecuciones de microbenchmarks en la página web, se documentará el formato de dicha información una vez que se diseñen los microbenchmarks y exáctamente cuáles datos son de más relevancia para cada uno.

## 5.6. Uso de patrones

No aplica.

## 5.7. Interfaces

- Interfaz de biblioteca core
- Interfaz de biblioteca microbenchmarks (No diseñada)
- API servidor

## 5.8. Interfaz de usuario

- CLI
- web

## 5.9. Estructura

No aplica

## 5.10. Interacción

No aplica.

## 5.11. Dinámica de estados

No aplica.

## 5.12. Algoritmos

Se documentará posteriorimente. Esta sección incluirá en detalle el diseño de los microbenchmarks creados.

## 5.13. Recursos

No aplica.

# 6. Apéndice - Alternativas de diseño

## 6.1. Creación de bindings/buffers

- Usar naga para inspeccionar shader y crear automáticamente

- Permitir múltiples bind groups

- Templating de shaders
