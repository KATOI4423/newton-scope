<!-- omit in toc -->
# 設計書

<!-- omit in toc -->
## 目次
- [GPU処理](#gpu処理)
  - [Floatによる高精度計算](#floatによる高精度計算)
    - [座標系の定義](#座標系の定義)
    - [Newton法の漸化式の変形](#newton法の漸化式の変形)
    - [Horner's rule](#horners-rule)
    - [関数と導関数を求める](#関数と導関数を求める)
    - [Horner's ruleのNewton法への適応](#horners-ruleのnewton法への適応)
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
z'_{n+1} = z'_n - \frac{\displaystyle \sum_{k=0}^{\infty} {\frac{s^k}{k!}f^k(c)z'^k_n}}{\displaystyle \sum_{k=0}^{\infty} {\frac{s^{k+1}}{k!}f^{k+1}(c)z'^k_n}}
\end{equation}
```

#### Horner's rule

Taylor展開を、最も少ない加算と乗算の演算回数で求めるために、Horner's ruleのアルゴリズムを用いる。

```math
\begin{equation}
f(z) = a_n z^n + a_{n-1} z^{n-1} + \cdots + a_1 z + a_0
\end{equation}
```

において、以下のように式変形することができる。

```math
\begin{equation}
f(z) = ( \cdots (a_n z + a_{n-1}) z + \cdots + a_1)z + a_0
\end{equation}
```

この変形した状態で計算すると、乗算を $n(n+1)/2$ 回から $n$ 回に減らすことができる。

#### 関数と導関数を求める

horner's ruleを使用して、関数 $f(z)$ と、その導関数 $f'(z)$ を求める。

まず、次の部分多項式を定義する。

```math
\begin{equation}
r_k(z) \overset{def}{=} \Big(\cdots (a_n z + a_{n-1}) z + \cdots + a_{k+1} \Big)
\end{equation}
```

すなわち、 $r_k$ は $a_{k+1}, a_{k+2}, \cdots, a_n$ のみを使用した多項式（Horner's ruleの途中値）である。

すると、部分的に $k$ 番目の項まで組み込んだ時の多項式 $f_k(z)$ は、

```math
\begin{equation}
f_k(z) = r_k(z)z + a_k
\end{equation}
```

となり、最終的には $f(z) = f_0(z)$ となる。

ここで、$f_k(z)$ を $z$ で微分すると、積の微分則から、以下の形になる。

```math
\begin{equation}
f'_k(z) = r'_k(z)z + r_k(z)
\end{equation}
```

つまり、部分多項式 $r_k$ の導関数 $r'_k$ を用いて、 $f'_k$ が計算できる。

$r_k$ の定義から、$r_k$ の漸化式は以下の通りになる。

```math
\begin{equation}
r_{k-1}(z) = r_k(z)z + a_k
\end{equation}
```

これを微分することで、 $r'_k$ についての漸化式を得る。

```math
\begin{equation}
r'_{k-1}(z) = r'_k(z)z + r_k(z)
\end{equation}
```

これらの漸化式を使用することで、Horner's ruleによって関数 $f(z)$ と、その導関数 $f'(z)$ を求めることができる。

#### Horner's ruleのNewton法への適応

式 $(8)$ をループで計算することを考える。

$s$ と $c$ はそれぞれ任意精度の浮動小数点型として与えるため、CPUで処理しなければならない。それとは異なり、$k!$ は整数型で計算できる値であり、 $z'^k_n$ はWebGLが保持するFloat型の座標値である。

そのため、$\{a_k\} := \{s^k f^k(c)\}$ をCPUで計算し、$z'^k_n / k!$ はGPUによって計算する。

```math
\begin{equation}
\begin{split}
\frac{z'^k_n}{k!} &= \frac{z'_n \cdot z'_n \cdots z'_n}{k (k-1) \cdots 3 \cdot 2 \cdot 1} \\
                  &= \frac{z'_n}{k} \frac{z'_n}{k-1} \cdots \frac{z'_n}{3} \frac{z'_n}{2} \frac{z'_n}{1} 
\end{split}
\end{equation}
```

となるので、Horner' ruleによるループ計算で、$z'_n$ の代わりに $z'_n / i$ （$i$ はループカウンタ）を使用することで対応できる。

```
coeffs = [..]; // coeff[0]~coeff[N-1]まで. coeff[i] = f^i * s^i
f  <- coeffs[N-1];
df <- 0;

for (i = N - 2; i >= 0; --i) {
    df <- df * z / (i + 1) + f
    f  <-  f * z / (i + 1) + coeffs[i];
}
```

#### 変形Newton法漸化式の精度

T.B.D.

