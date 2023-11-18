use euclid::{vec3, Angle, Rotation3D, Vector3D};
use std::f64::consts::PI;
use std::time::{SystemTime, UNIX_EPOCH};

enum U {}

const INITIAL_PHASE: f64 = 1.7247443415579253;
const SIDEREAL: f64 = 31558144.36363983;

fn get_phase(now: f64, initial_phase: f64, sidereal: f64) -> Angle<f64> {
    Angle::radians((initial_phase + (now / sidereal * 2.0 * PI) % (2.0 * PI)) % (2.0 * PI))
}

fn get_sun_direction(phase: Angle<f64>) -> Vector3D<f64, U> {
    -Rotation3D::around_z(phase).transform_vector3d(vec3::<_, U>(1.0, 0.0, 0.0))
}

pub fn get_sun_position() -> (f64, f64) {
    let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let now = since_the_epoch.as_secs() as f64 + since_the_epoch.subsec_nanos() as f64 * 1e-9;

    let phase = get_phase(now, INITIAL_PHASE, SIDEREAL);
    let to_sun = get_sun_direction(phase);
    dbg!(to_sun);

    (1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_phase() {
        assert!((get_phase(0.0, INITIAL_PHASE, SIDEREAL).radians - INITIAL_PHASE).abs() < 10.0);
        assert!((get_phase(22895580.0, INITIAL_PHASE, SIDEREAL).radians - 0.0).abs() < 10.0);
        assert!((get_phase(811849260.0, INITIAL_PHASE, SIDEREAL).radians - 0.0).abs() < 10.0);
        assert!((get_phase(1600802520.0, INITIAL_PHASE, SIDEREAL).radians - 0.0).abs() < 10.0);
    }

    #[test]
    fn test_get_sun_direction() {
        assert!(
            (get_sun_direction(Angle::radians(0.0)) - vec3::<_, U>(-1.0, 0.0, 0.0)).length() < 1e-9
        );
        assert!(
            (get_sun_direction(Angle::radians(PI / 2.0)) - vec3::<_, U>(0.0, -1.0, 0.0)).length()
                < 1e-9
        );
        assert!(
            (get_sun_direction(Angle::radians(PI)) - vec3::<_, U>(1.0, 0.0, 0.0)).length() < 1e-9
        );
        assert!(
            (get_sun_direction(Angle::radians(3.0 * PI / 2.0)) - vec3::<_, U>(0.0, 1.0, 0.0))
                .length()
                < 1e-9
        );
    }
}
