#!/usr/bin/env python3

import numpy as np
from numpy.fft import fft,ifft
import matplotlib.pyplot as plt

from scipy.signal import fftconvolve

a = np.zeros(8)
b = np.zeros(8)
a[6] = 1.0
b[5] = 1.0

print(a,'a')
print(b,'b')
c = fftconvolve(a,b[::-1])
c[np.abs(c) < 1e-15] = 0.0
print(c,'fftconvolve')

print(np.correlate(a,b,mode='full'),'corr')

n = len(a)+len(b)-1
a2 = np.zeros(n)
b2 = np.zeros(n)
a2[:len(a)] = a
b2[:len(b)] = b[::-1]
c2 = ifft(fft(a2) * fft(b2)).real
c2[np.abs(c2) < 1e-15] = 0.0
print(c2,'iF(F(a)*F(b)')

c3 = ifft(fft(a,n) * fft(b[::-1],n)).real
c3[np.abs(c3) < 1e-15] = 0.0
print(c3,'iF(F(a)*F(b)')
print(np.argmax(c3) - len(c3)//2)

