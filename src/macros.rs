


macro_rules! f32_undef {
    ($s:ident, $q:ident, $t:ty, $($x:ident),*) => ( $( $s.$x = SAC_FLOAT_UNDEF; )* );
}
macro_rules! i32_undef {
    ($s:ident, $q:ident, $t:ty, $($x:ident),*) => ( $( $s.$x = SAC_INT_UNDEF; )* );
}
macro_rules! str_undef {
    ($s:ident, $q:ident, $($x:ident),*) => ( $( $s.$x = String::from("-12345  "); )* );
}
macro_rules! u8s_undef {
    ($s:ident, $q:ident, $($x:ident),*) => ( $(
        for i in 0 .. $s.$x.len() {
            $s.$x[i] = 32;
        }
        $s.$x[0] = 45;
        $s.$x[1] = 49;
        $s.$x[2] = 50;
        $s.$x[3] = 51;
        $s.$x[4] = 52;
        $s.$x[5] = 53;
    )* );
}

macro_rules! sac_reals {
    ($s:ident, $function:ident) => { sac_reals!($s, ignore_idnet, ignore_type, $function); };
    ($s:ident, $z:ident, $function:ident) => { sac_reals!($s, $z, ignore_type, $function); };
    ($s:ident, $z:ident, $t:ty, $function:ident) => {
        $function!($s,$z,$t,
                   delta, depmin, depmax, scale, odelta, b, e, o, a, fmt,
                   t0, t1, t2, t3, t4, t5, t6, t7, t8, t9, f,
                   resp0, resp1, resp2, resp3, resp4,
                   resp5, resp6, resp7, resp8, resp9,
                   stla, stlo, stel, stdp, evla, evlo, evel, evdp, mag,
                   user0, user1, user2, user3, user4,
                   user5, user6, user7, user8, user9,
                   dist, az, baz, gcarc, sb, sdelta,
                   depmen, cmpaz, cmpinc,
                   xminimum, xmaximum, yminimum, ymaximum,
                   unused6, unused7, unused8, unused9, unused10,
                   unused11, unused12
        );
    }
}


macro_rules! sac_ints {
    ($s:ident, $function:ident) => { sac_ints!($s, ignore_idnet, ignore_type, $function); };
    ($s:ident, $z:ident, $function:ident) => { sac_ints!($s, $z, ignore_type, $function); };
    ($s:ident, $z:ident, $t:ty, $function:ident) => {
        $function!($s,$z,$t,
                   nzyear, nzjday, nzhour, nzmin, nzsec, nzmsec, nvhdr,
                   norid, nevid, npts, nsnpts, nwfid,
                   nxsize, nysize, unused15, iftype, idep, iztype,
                   unused16, iinst, istreg, ievreg, ievtyp,
                   iqual, isynth, imagtyp, imagsrc,
                   unused19, unused20, unused21, unused22,
                   unused23, unused24, unused25, unused26,
                   leven, lpspol, lovrok, lcalda, unused27);

    };
}

macro_rules! sac_strings {
    ($s:ident, $function:ident) => { sac_strings!($s, ignore_ident, $function); };
    ($s:ident, $x:ident, $function:ident) => {
        $function!($s,$x,
                   kstnm, kevnm, khole, ko, ka,
                   kt0, kt1, kt2, kt3, kt4, kt5, kt6, kt7, kt8, kt9,
                   kf, kuser0, kuser1, kuser2, kcmpnm, knetwk, kdatrd, kinst);
    }
}

macro_rules! sac_u8_strings {
    ($s:ident, $function:ident) => { sac_u8_strings!($s, ignore_ident, $function); };
    ($s:ident, $z:ident, $function:ident) => {
        $function!($s,$z,
                   u8_kstnm, u8_kevnm, u8_khole, u8_ko, u8_ka,
                   u8_kt0, u8_kt1, u8_kt2, u8_kt3, u8_kt4,
                   u8_kt5, u8_kt6, u8_kt7, u8_kt8, u8_kt9,
                   u8_kf, u8_kuser0, u8_kuser1, u8_kuser2, u8_kcmpnm,
                   u8_knetwk, u8_kdatrd, u8_kinst);
    }
}


macro_rules! string_to_u8 {
    ($s:ident, $($a:ident, $b:ident),*) => {
        $(
            let mut tmp = if $s.$b.len() == 8 {
                format!("{:8}", $s.$a)
            } else {
                format!("{:16}", $s.$a)
            };
            tmp.truncate($s.$b.len());
            if tmp.trim_end().len() == 0 {
                tmp = format!("{:8}", "-12345");
            }
            $s.$b.copy_from_slice( tmp.as_bytes() );
        )*
    }
}

macro_rules! sac_strings_pair {
    ($s:ident, $function:ident) => {
        $function!($s,
                   kstnm, u8_kstnm, kevnm, u8_kevnm, khole, u8_khole,
                   ko, u8_ko, ka, u8_ka,
                   kt0, u8_kt0, kt1, u8_kt1, kt2, u8_kt2, kt3, u8_kt3, kt4, u8_kt4,
                   kt5, u8_kt5, kt6, u8_kt6, kt7, u8_kt7, kt8, u8_kt8, kt9, u8_kt9,
                   kf, u8_kf,
                   kuser0, u8_kuser0, kuser1, u8_kuser1, kuser2, u8_kuser2,
                   kcmpnm, u8_kcmpnm, knetwk,  u8_knetwk, kdatrd, u8_kdatrd,
                   kinst, u8_kinst
        );
    }
}

macro_rules! write_real {
    ($s:ident, $fp:ident, $t:ty, $x:ident) => ( $fp.write_f32::<$t>($s.$x)?; );
}
macro_rules! write_reals {
    ($s:ident, $fp:ident, $t:ty, $($x:ident),+) => ( $( write_real!($s,$fp,$t,$x); )+ );
}
macro_rules! write_int {
    ($s:ident, $fp:ident, $t:ty, $x:ident) => ( $fp.write_i32::<$t>($s.$x)?; );
}
macro_rules! write_ints {
    ($s:ident, $fp:ident, $t:ty, $($x:ident),+) => ( $( write_int!($s,$fp,$t,$x); )+ );
}


macro_rules! read_real {
    ($s:ident, $fp:ident, $t:ty, $x:ident) => ( $s.$x = $fp.read_f32::<$t>()?; );
}
macro_rules! read_reals {
    ($s:ident, $fp:ident, $t:ty, $($x:ident),+) => ( $( read_real!($s,$fp,$t,$x); )+ );
}
macro_rules! read_int {
    ($s:ident, $fp:ident, $t:ty, $x:ident) => ( $s.$x = $fp.read_i32::<$t>()?; );
}
macro_rules! read_ints {
    ($s:ident, $fp:ident, $t:ty, $($x:ident),+) => ( $( read_int!($s,$fp,$t,$x); )+ );
}


macro_rules! read_strings {
    ($s:ident, $fp:ident, $($x:ident),+) => ( $( $fp.read_exact(&mut $s.$x)?; )+ );
}
macro_rules! write_strings {
    ($s:ident, $fp:ident, $($x:ident),+) => ( $( $fp.write_all(&$s.$x)?; )+ );
}
macro_rules! u8_to_string {
    ($s:ident, $($x:ident, $u8x:ident),*) => (
        $( $s.$x = String::from_utf8($s.$u8x.to_vec()).unwrap(); )*
    );
}
