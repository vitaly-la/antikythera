use euclid::{vec3, Angle, Rotation3D, Vector3D};
use std::f64::consts::PI;

enum U {}

const INITIAL_PHASE: f64 = 1.7247443415579253;
const SIDEREAL: f64 = 31558144.36363983;

const INITIAL_DAILY_PHASE: f64 = 1.736602605734358;
const SIDEREAL_DAY: f64 = 86164.0905;

const AXIAL_TILT: f64 = 23.44 * PI / 180.0;
const AXIAL_DIRECTION: f64 = 1.54075846982669;

const LAT: f64 = 51.5072 * PI / 180.0;
const LON: f64 = 0.0;

const INITIAL_MOON_PHASE: f64 = 5.0;
const SIDEREAL_MONTH: f64 = 27.321661547 * 24.0 * 60.0 * 60.0;

fn to_local_coords(lat: f64, lon: f64, vec: Vector3D<f64, U>) -> Vector3D<f64, U> {
    Rotation3D::around_z(Angle::radians(lon))
        .transform_vector3d(Rotation3D::<_, _, U>::around_y(-Angle::radians(lat)).transform_vector3d(vec))
}

fn to_recent_coords(daily_phase: Angle<f64>, vec: Vector3D<f64, U>) -> Vector3D<f64, U> {
    Rotation3D::around_z(daily_phase).transform_vector3d(vec)
}

fn to_global_coords(axial_tilt: f64, axial_direction: f64, vec: Vector3D<f64, U>) -> Vector3D<f64, U> {
    Rotation3D::around_z(Angle::radians(axial_direction)).transform_vector3d(
        Rotation3D::<_, _, U>::around_y(Angle::radians(axial_tilt)).transform_vector3d(
            Rotation3D::<_, _, U>::around_z(-Angle::radians(axial_direction)).transform_vector3d(vec),
        ),
    )
}

fn get_phase(ts: f64, initial_phase: f64, sidereal: f64) -> Angle<f64> {
    Angle::radians((initial_phase + (ts / sidereal * 2.0 * PI) % (2.0 * PI)) % (2.0 * PI))
}

fn get_sun_direction(phase: Angle<f64>) -> Vector3D<f64, U> {
    -Rotation3D::around_z(phase).transform_vector3d(vec3::<_, U>(1.0, 0.0, 0.0))
}

fn get_moon_direction(moon_phase: Angle<f64>) -> Vector3D<f64, U> {
    Rotation3D::around_z(-moon_phase).transform_vector3d(vec3::<_, U>(1.0, 0.0, 0.0))
}

fn get_altitude(normal: Vector3D<f64, U>, to_sun: Vector3D<f64, U>) -> f64 {
    PI / 2.0 - normal.dot(to_sun).acos()
}

fn get_azimuth(normal: Vector3D<f64, U>, north: Vector3D<f64, U>, to_sun: Vector3D<f64, U>) -> f64 {
    let proj = (to_sun - normal * normal.dot(to_sun)).normalize();
    let angle = north.dot(proj).acos();
    let east = north.cross(normal);
    if east.dot(proj) > 0.0 {
        angle
    } else {
        2.0 * PI - angle
    }
}

fn get_normal_and_north(ts: f64) -> (Vector3D<f64, U>, Vector3D<f64, U>) {
    let daily_phase = get_phase(ts, INITIAL_DAILY_PHASE, SIDEREAL_DAY);
    let normal = to_global_coords(
        AXIAL_TILT,
        AXIAL_DIRECTION,
        to_recent_coords(daily_phase, to_local_coords(LAT, LON, vec3::<_, U>(1.0, 0.0, 0.0))),
    );
    let north = to_global_coords(
        AXIAL_TILT,
        AXIAL_DIRECTION,
        to_recent_coords(daily_phase, to_local_coords(LAT, LON, vec3::<_, U>(0.0, 0.0, 1.0))),
    );
    (normal, north)
}

pub fn get_sun_position(ts: f64) -> (f64, f64) {
    let (normal, north) = get_normal_and_north(ts);

    let phase = get_phase(ts, INITIAL_PHASE, SIDEREAL);
    let to_sun = get_sun_direction(phase);

    let alt = get_altitude(normal, to_sun);
    let az = get_azimuth(normal, north, to_sun);

    (alt, az)
}

pub fn get_moon_position(ts: f64) -> (f64, f64) {
    let (normal, north) = get_normal_and_north(ts);

    let moon_phase = get_phase(ts, INITIAL_MOON_PHASE, SIDEREAL_MONTH);
    let to_moon = get_moon_direction(moon_phase);

    let alt = get_altitude(normal, to_moon);
    let az = get_azimuth(normal, north, to_moon);

    (alt, az)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_local_coords() {
        let normal = vec3::<_, U>(1.0, 0.0, 0.0);
        assert!((to_local_coords(0.0, 0.0, normal) - vec3::<_, U>(1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((to_local_coords(PI / 2.0, 0.0, normal) - vec3::<_, U>(0.0, 0.0, 1.0)).length() < 1e-15);
        assert!((to_local_coords(-PI / 2.0, 0.0, normal) - vec3::<_, U>(0.0, 0.0, -1.0)).length() < 1e-15);
        assert!((to_local_coords(0.0, PI / 2.0, normal) - vec3::<_, U>(0.0, 1.0, 0.0)).length() < 1e-15);
        assert!((to_local_coords(0.0, PI, normal) - vec3::<_, U>(-1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((to_local_coords(0.0, -PI / 2.0, normal) - vec3::<_, U>(0.0, -1.0, 0.0)).length() < 1e-15);
    }

    #[test]
    fn test_to_recent_coords() {
        let normal = vec3::<_, U>(1.0, 0.0, 0.0);
        assert!((to_recent_coords(Angle::radians(0.0), normal) - vec3::<_, U>(1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((to_recent_coords(Angle::radians(PI / 2.0), normal) - vec3::<_, U>(0.0, 1.0, 0.0)).length() < 1e-15);
    }

    #[test]
    fn test_to_global_coords() {
        let axis = vec3::<_, U>(0.0, 0.0, 1.0);
        assert!((to_global_coords(0.0, 0.0, axis) - vec3::<_, U>(0.0, 0.0, 1.0)).length() < 1e-15);
        assert!((to_global_coords(PI / 2.0, 0.0, axis) - vec3::<_, U>(1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((to_global_coords(PI / 2.0, PI / 2.0, axis) - vec3::<_, U>(0.0, 1.0, 0.0)).length() < 1e-15);
        assert!((to_global_coords(PI / 2.0, -PI / 2.0, axis) - vec3::<_, U>(0.0, -1.0, 0.0)).length() < 1e-15);
    }

    #[test]
    fn test_get_phase() {
        assert!((get_phase(0.0, INITIAL_PHASE, SIDEREAL).radians - INITIAL_PHASE).abs() < 1e-4);
        assert!(get_phase(22895580.0, INITIAL_PHASE, SIDEREAL).radians.abs() < 1e-4);
        assert!(get_phase(811849260.0, INITIAL_PHASE, SIDEREAL).radians.abs() < 1e-4);
        assert!((get_phase(1600802520.0, INITIAL_PHASE, SIDEREAL).radians - 2.0 * PI).abs() < 1e-4);
    }

    #[test]
    fn test_get_sun_direction() {
        assert!((get_sun_direction(Angle::radians(0.0)) - vec3::<_, U>(-1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((get_sun_direction(Angle::radians(PI / 2.0)) - vec3::<_, U>(0.0, -1.0, 0.0)).length() < 1e-15);
        assert!((get_sun_direction(Angle::radians(PI)) - vec3::<_, U>(1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((get_sun_direction(Angle::radians(3.0 * PI / 2.0)) - vec3::<_, U>(0.0, 1.0, 0.0)).length() < 1e-15);
    }

    #[test]
    fn test_get_moon_direction() {
        assert!((get_moon_direction(Angle::radians(PI / 2.0)) - vec3::<_, U>(0.0, -1.0, 0.0)).length() < 1e-15);
    }

    #[test]
    fn test_get_altitude() {
        assert!((get_altitude(vec3::<_, U>(1.0, 0.0, 0.0), vec3::<_, U>(1.0, 0.0, 0.0)) - PI / 2.0).abs() < 1e-15);
        assert!((get_altitude(vec3::<_, U>(1.0, 0.0, 0.0), vec3::<_, U>(0.0, 1.0, 0.0)) - 0.0).abs() < 1e-15);
        assert!((get_altitude(vec3::<_, U>(1.0, 0.0, 0.0), vec3::<_, U>(-1.0, 0.0, 0.0)) + PI / 2.0).abs() < 1e-15);
    }

    #[test]
    fn test_get_azimuth() {
        assert!(
            (get_azimuth(
                vec3::<_, U>(0.0, 0.0, 1.0),
                vec3::<_, U>(0.0, 1.0, 0.0),
                vec3::<_, U>(0.0, 0.6, 0.8)
            ) - 2.0 * PI)
                .abs()
                < 1e-15
        );
        assert!(
            (get_azimuth(
                vec3::<_, U>(0.0, 0.0, 1.0),
                vec3::<_, U>(0.0, 1.0, 0.0),
                vec3::<_, U>(1.0, 0.0, 0.0)
            ) - PI / 2.0)
                .abs()
                < 1e-15
        );
    }
}
