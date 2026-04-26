# 設計書

## Perturbation

### 背景

ニュートンフラクタル描画では、各ピクセルごとに以下の反復計算を行う.

```math
\begin{equation}
z_{n+1} = z_n - \frac{f(z_n)}{f'(z_n)}, \quad z_0 = c
\end{equation}
```

深いズーム領域では、以下の問題が生じる.

- 隣接ピクセル間の `c` の差が極めて小さい.
- 収束判定は非常に敏感.
- 高精度（多倍長）演算が必要になるが、各ピクセルで演算すると極めて遅い.


### 基本アイデア

Preturbation法では、以下のように計算を行う.

- 基準点 $c_0$ を1つ選び、その軌道を高精度で計算する.
- 他の点 $c = c_0 + \delta c$ は差分 $\delta z$ を用いて低精度で計算する.


### 数式展開

ニュートン写像を

```math
\begin{equation}
N(z) := z - \frac{f(z)}{f'(z)}
\end{equation}
```

と定義する.


#### 基準点の軌道

```math
\begin{equation}
z_{n+1}|_{c=c_0} = N(z_n|_{c=c_0})
\end{equation}
```


#### 対象点の軌道

```math
\begin{equation}
z_n|_{c=c_0 + \delta c} = z_n|_{c=c_0} + \delta z_n|_{c=c_0 + \delta c}
\end{equation}
```


### 1次近似

テイラー展開により、

```math
\begin{equation}
\begin{split}
z_{n+1}|_{c=c_0 + \delta c} &= N(z_n|_{c=c_0} + \delta z) \\
    &\approx N(z_n|_{c=c_0}) + \delta z_n|_{c=c_0} \times N'(z_n|_{c=c_0})
\end{split}
\end{equation}
```

となるので、

```math
\begin{equation}
\delta z_{n+1}|_{c=c_0 + \delta c} = \delta z_n|_{c=c_0} \times \Big( 1 - \frac{f(z_n|_{c=c_0})f'(z_n|_{c=c_0})}{f''(z_n|_{c=c_0})^2} \Big)
\end{equation}
```

### $\delta c$ の扱い

ニュートン法では、 $\delta z$ の初期条件としてのみ影響する.

```math
\begin{equation}
\delta z_0|_{c=c_0 + \delta c} = \delta c
\end{equation}
```

## スケーリング

### 目的

深いズームでは pixel size $s$ が極めて小さくなり、 $\delta z$ が f64 の非正規化数領域（$< 2.2 \times 10^{-308}$）に達する危険がある。
非正規化数領域では有効ビット数が減少し、精度が劣化する。

$s$ を計算から分離し、 $\hat{z}$ を常に f64 が正確に表現できるスケールに保つことで、これを回避する。

### 定義

pixel size を $s$、ピクセル座標（整数オフセット）を $u$ として、

```math
\begin{equation}
\delta z_n = s \cdot \hat{z}_n, \quad \delta c = s \cdot u
\end{equation}
```
と分離する。

### 更新式

$\delta z_{n+1} = N'(z_n) \cdot \delta z_n$ に代入すると、

```math
\begin{equation}
\hat{z}_{n+1} = N'(z_n) \cdot \hat{z}_n, \quad \hat{z}_0 = u
\end{equation}
```

となり、$s$ は計算から完全に分離される。

### 精度上の根拠

| 状況 | 問題 | スケーリングの効果 |
| -- | -- | -- |
| $s \ll 10^{-100}$ | $\delta z = s \cdot u$ が非正規化数域に入る | $\hat{z}_0 = u$ （ピクセル座標）は正規化数域を保つ |
| $\|N'\| < 1$ が続く | $|\delta z\|$ が指数減少し非正規化数域に達する | $\hat{z}$ が正規化数域を保つ限り有効ビット52桁を維持 |
| rebase 時の加算 | $\delta z$ が極小だと $\delta z + \delta c$ で桁落ち | $\hat{z} + u$ は適切なスケールで加算できる |

$\hat{z}_0 = u$ （典型的に $|u| \sim 10〜10^3$）は f64 が最も正確に表現できる領域に近く、非正規化数・オーバーフローの両方から十分な余裕がある。

### 限界

$|N'(z_n)| \gg 1$ となる領域（$f'(z_n) \approx 0$ 付近、すなわち $f$ の臨界点近傍）では $|\hat{z}|$ が発散し、スケーリングで精度を保つことはできない。この場合は rebase（基準点の再選択）が必要となる。

### Rebase
$|\hat{z}_n|$ が閾値を超えた場合、基準点 $c_0$ を現在の対象点に更新し、 $\delta z$ をリセットする。

```math
\begin{equation}
c_0 \leftarrow c_0 + \delta z_n, \quad \delta z_n \leftarrow 0
\end{equation}
```
