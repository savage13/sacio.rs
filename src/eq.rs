
use crate::Sac;

macro_rules! xeq {
    ($a:ident,$b:ident,$t:ty,$($x:ident),*) => {
        $( if $a.$x != $b.$x {
            println!("field {}:  {} != {}", stringify!($x),$a.$x, $b.$x);
            return false;
        } )*
    };
}
macro_rules! xeqf {
    ($a:ident,$b:ident,$t:ty,$($x:ident),*) => {
        $( if ($a.$x - $b.$x).abs() > 1e-5 {
            let dx = ($a.$x - $b.$x).abs();
            println!("field {}: {} != {} [{}]", stringify!($x),$a.$x, $b.$x, dx);
            return false;
        } )*
    };
}

fn veq(a: &[f32], b: &[f32], tol: f32) -> bool {
    if a.len() != b.len() {
        println!("Data Lenghts unequal: {} vs {}", a.len(), b.len());
        return false;
    }
    if a != b {
        for i in 0 .. a.len() {
            println!("{:6} {:21.15e} {:21.15e} {:21.15e}", i, a[i], b[i], (a[i]-b[i]).abs());
            if (a[i] - b[i]).abs() > tol {
                println!("{}: {} {} tol: {}", i, a[i], b[i], tol);
                return false;
            }
        }
        return true;
    }
    true
}

impl PartialEq for Sac {
    fn eq(&self, other: &Sac) -> bool {
        //println!("eq ints");
        sac_ints!(self,    other, xeq);
        //println!("eq strings");
        sac_strings!(self, other, xeq);
        //println!("eq reals");
        sac_reals!(self,   other, xeqf);
        //println!("eq npts");
        if self.npts != other.npts {
            //println!("npts not equal {} {}",self.npts, other.npts);
            return false;
        }
        //println!("y len");
        if self.y.len() != other.y.len() {
            //println!("npts not equal in vec, :/ {} {}", self.y.len(), other.y.len());
            return false;
        }
        //println!("y compare {}", self.y.len());
        veq(&self.y, &other.y, 1e-5) &&
            veq(&self.x, &other.x, 1e-5)
    }
}
