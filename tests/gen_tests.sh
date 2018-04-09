#!/bin/sh
sac -d <<EOF
fg impulse npts 100 delta 0.1
ch lhdr5 no
write imp.sac

fg impulse npts 100 delta 0.1
ch lhdr5 no
fft rlim
write imp_fft_rlim.sac

fg impulse npts 100 delta 0.1
ch lhdr5 no
fft amph
write imp_fft_amph.sac

fg impulse npts 100 delta 0.1
ch lhdr5 no
fft rlim
ifft
write imp_fft_ifft.sac

fg sine 0.05 0.0 npts 100 delta 1
ch lhdr5 no
write sine.sac

fg sine 0.05 0.0 npts 100 delta 1
ch lhdr5 no
fft rlim
write sine_fft_rlim.sac

fg sine 0.05 0.0 npts 100 delta 1
ch lhdr5 no
fft amph
write sine_fft_amph.sac

fg sine 0.05 0.0 npts 100 delta 1
ch lhdr5 no
fft rlim
ifft
write sine_fft_ifft.sac

fg seismo
ch lhdr5 no
write file.sac

fg sin 0.01 0.5 npts 100 delta 100
ch lhdr5 no
write sine2.sac
fft amph
write sine2_fft_amph.sac

fg impulse npts 1000 begin 0 delta 0.1
ch lhdr5 no
write imp1000.sac
hilbert
write hilbert_imp.sac

read imp1000.sac
envelope
write envelope_imp.sac

read imp.sac imp.sac
correlate
dc 1
write correlate_imp.sac

fg random 1 1 npts 100 delta 1.0
write rand1.sac
fg random 1 2 npts 100 delta 1.0
write rand2.sac
read rand1.sac rand2.sac
correlate
dc 1
write correlate_rand.sac

fg random 1 1 npts 100 delta 1.0 begin 10
write rand1b.sac
fg random 1 2 npts 100 delta 1.0 begin 20
write rand2b.sac
read rand1b.sac rand2b.sac
correlate
dc 1
write correlate_rand_b.sac

fg boxcar npts 20 delta 1.0
write boxcar.sac
read rand1.sac boxcar.sac
convolve
dc 1
write convolve_boxcar.sac

read rand1.sac
rtrend
write rand1_rtr.sac

fg seismo
write seismo.sac
rtr
write seismo_rtr.sac

read seismo.sac
taper type hanning width 0.05
write seismo_taper_han.sac

read seismo.sac
taper type hamming width 0.05
write seismo_taper_ham.sac

read seismo.sac
taper type cosine width 0.05
write seismo_taper_cos.sac

read seismo.sac
rmean
write seismo_rmean.sac

read rand1.sac
rmean
write rand1_rmean.sac

read seismo.sac
reverse
write seismo_reverse.sac

read imp.sac
bp co 0.1 0.5 p 2 n 4
write imp_bp_0.1_0.5.sac

read imp.sac
lp co 0.5 p 2 n 4
write imp_lp_0.5.sac

read imp.sac
br co 0.1 0.5 p 2 n 4
write imp_br_0.1_0.5.sac

read imp.sac
hp co 0.5 p 2 n 4
write imp_hp_0.5.sac

quit
EOF

sacswap file.sac
mv file.sac file.swp.sac
