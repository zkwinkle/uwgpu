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
microbenchmarks de forma nativa compilando a APIs gráficas como Metal, Vulkan y
DX12 mediante el API gráfico _wgpu_. Este documento especifica los requisitos
funcionales, las características del usuario, el alcance del producto, y las
verificaciones.

## 1.1. Propósito

El propósito principal del software es proporcionar una suite de
microbenchmarks para medir características específicas del rendimiento de GPUs
en plataformas que soportan WebGPU y otras APIs gráficas.

También facilita la ejecución de microbenchmarks propios de los usuarios.

Este software aborda la necesidad de obtener datos de rendimiento detallados y
comparativos a través de diferentes combinaciones de hardware y plataformas,
permitiendo a desarrolladores e investigadores optimizar su software para
diferentes y evaluar el desempeño general de las GPUs.

## 1.2. Alcance

El alcance del software, denominado "µwgpu compute" abarca el diseño,
desarrollo e implementación de los microbenchmarks para evaluar características
del hardware de GPUs y las interfaces para ejecutarlos y visualizar resultados.

El software incluirá dos interfaces principales:

1. Interfaz web: Permite ejecutar microbenchmarks desde un navegador,
   recolectar datos, visualizar resultados, y proporcionar una opción para
   ejecutar microbenchmarks personalizados.
2. Interfaz CLI: Permite ejecutar microbenchmarks nativamente, compilando para
   APIs gráficas como Metal, Vulkan y DX12, y ofrece resultados en texto.

 El producto es autónomo y no interactúa con sistemas externos, aunque opera
 dentro del contexto los APIs gráficos mencionados y las plataformas sobre las
 que se ejecutan.

## 1.3. Resumen del producto

### 1.3.1. Perspectiva del producto

Los microbenchmarks se implementarán utilizando el API de gráficos _wgpu_ que
se puede correr en browsers que tengan soporta para WebGPU o nativamente
traduciendo a los APIs nativos Vulkan, Metal y DX12.

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

### 1.3.3. Características del usuario

Los usuarios esperados incluyen desarrolladores de software de gráficos por
computadora e investigadores en GPU compute u otros temas de rendimiento en
GPU. Se espera que los usuarios tengan conocimientos sobre WebGPU y APIs
gráficas, así como habilidades para interpretar resultados de benchmarks y
optimizar el rendimiento de software para GPUs.

### 1.3.4. Limitaciones

No hay limitaciones significativas que afecten el desarrollo del producto. El
software no enfrenta restricciones de hardware o software específicas, y el
tiempo disponible para el proyecto se ha considerado al definir el alcance.

## 1.4. Definiciones

- **Microbenchmark:** Prueba que mide el rendimiento de alguna característica específica de bajo nivel. Se diferencia de un "benchmark" ya que ese es un concepto más general que aplica a cualquier tipo de prueba que podría incluir el rendimiento de sistemas enteros.
- **GPU Compute:** Uso de la GPU para cálculos computacionales de manera general.

## 1.5. Siglas y abreviaturas

**GPU:** Unidad de procesamiento gráfico.
**CLI:** Interfaz de línea de comandos.

# 2. Referencias

# 3. Requisitos

## 3.1. Funciones

## 3.2. Requisitos de usabilidad

## 3.3. Interfaces externas

## 3.4. Requisitos de rendimiento

## 3.5. Requisitos de la base de datos lógica

No aplica.

## 3.6. Restricciones de diseño

## 3.7. Atributos del sistema de software

## 3.8. Información de soporte

# 4. Verificación

# 5. Apéndices

## 5.1. Supuestos y dependencias

## 5.2. Matriz de trazabilidad
