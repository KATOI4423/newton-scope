<!-- omit in toc -->
# 設計書

<!-- omit in toc -->
## 目次
- [GPU処理](#gpu処理)
  - [Floatによる高精度計算](#floatによる高精度計算)
    - [座標系の定義](#座標系の定義)
    - [Newton法の漸化式の変形](#newton法の漸化式の変形)
    - [変形Newton法漸化式の精度](#変形newton法漸化式の精度)

## GPU処理

### Floatによる高精度計算

WebGLでの浮動小数点演算では、Float精度しか使用できない。
そのため、そのままFloatにて計算を行うと、ズーム倍率の上限が著しく低下する。

本アプリでは、`Taylor展開`で`f(z)`を展開し、`CPUでの高精度浮動小数点演算による係数計算`と`GPUのFloat演算による高速Pixel計算`を両立させる。

#### 座標系の定義

まず、実際の複素数平面の座標

```math
\begin{equation}
z = x + iy
\end{equation}
```

に対して、

```math
\begin{equation}
z = c + s z
\end{equation}
```

となる 中心座標 $c$、ズーム倍率 $s$、総体座標 $z'$ を導入する。

$z'$ は、**$c$ を中心にズーム倍率 $s$ で拡大表示した座標** ということになる。

#### Newton法の漸化式の変形

Newton法の漸化式

```math
\begin{equation}
x_{n+1} = x_n - \frac{f(x_n)}{f'(x_n)}
\end{equation}
```

に 式$(2)$を代入することで、次の式を得る。

```math
\begin{equation}
\begin{split}
c + sz'_{n+1} &= c + sz'_n - \frac{f(c + sz'_n)}{f'(c + sz'_n)} \\
\Rightarrow z'_{n+1} &= z'_n - \frac{f(c + sz'_n)}{sf'(c + sz'_n)}
\end{split}
\end{equation}
```

ここで、Taylor展開

```math
\begin{equation}
f(ax+b) = \sum_{n=0}^{\infty} {\frac{a^n}{n!}f^n(b)x^n}
\end{equation}
```

を使用して、

```math
\begin{equation}
f(c + sz'_n) = \sum_{k=0}^{\infty} {\frac{s^k}{k!}f^k(c)z'^k_n}
\end{equation}
```

```math
\begin{equation}
f'(c + sz'_n) = \sum_{k=0}^{\infty} {\frac{s^k}{k!}f^{k+1}(c)z'^k_n}
\end{equation}
```

を得る。これを式$(4)$に代入して、Taylor展開によるNewton法の漸化式を得る。

```math
\begin{equation}
z'_{n+1} = z'_n - \frac{\displaystyle \sum_{k=0}^{\infty} {\frac{s^k}{k!}f^k(c)z'^k_n}}{\displaystyle s\sum_{k=0}^{\infty} {\frac{s^k}{k!}f^{k+1}(c)z'^k_n}}
\end{equation}
```

#### 変形Newton法漸化式の精度

T.B.D.

