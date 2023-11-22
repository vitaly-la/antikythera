use euclid::{vec3, Angle, Rotation3D, Vector3D};
use std::f64::consts::PI;

enum U {}

pub struct Star {
    pub ascension: f64,
    pub declination: f64,
    pub magnitude: f64,
}

pub struct Engine {
    ts: f64,
    normal: Vector3D<f64, U>,
    north: Vector3D<f64, U>,
}

const INITIAL_PHASE: f64 = 1.7247432929978155; // average from horizons
const SIDEREAL: f64 = 365.256363004 * 24.0 * 60.0 * 60.0; // from stellarium

const INITIAL_DAILY_PHASE: f64 = 1.7341591447815659; // from actual solar noon
const SIDEREAL_DAY: f64 = 23.9344694 * 60.0 * 60.0; // from stellarium

const AXIAL_TILT: f64 = 23.4392803055555555556 * PI / 180.0; // from stellarium
const AXIAL_DIRECTION: f64 = 1.5407643946374219; // average solstice

const LAT: f64 = 51.4775 * PI / 180.0; // greenwich
const LON: f64 = 0.0; // greenwich

const INITIAL_MOON_PHASE: f64 = 3.475;
const SIDEREAL_MONTH: f64 = 27.321582 * 24.0 * 60.0 * 60.0; // from stellarium

const MOON_INCLINATION: f64 = 5.145396 * PI / 180.0; // from stellarium
const INITIAL_NODAL_PHASE: f64 = 0.0;
const NODAL_PERIOD: f64 = 18.6 * 365.0 * 24.0 * 60.0 * 60.0;

fn rot_y(angle: f64, vec: Vector3D<f64, U>) -> Vector3D<f64, U> {
    Rotation3D::around_y(Angle::radians(angle)).transform_vector3d(vec)
}

fn rot_z(angle: f64, vec: Vector3D<f64, U>) -> Vector3D<f64, U> {
    Rotation3D::around_z(Angle::radians(angle)).transform_vector3d(vec)
}

fn get_phase(ts: f64, initial_phase: f64, sidereal: f64) -> f64 {
    (initial_phase + (ts / sidereal * 2.0 * PI) % (2.0 * PI)) % (2.0 * PI)
}

fn to_local_coords(lat: f64, lon: f64, vec: Vector3D<f64, U>) -> Vector3D<f64, U> {
    rot_z(lon, rot_y(-lat, vec))
}

fn to_recent_coords(daily_phase: f64, vec: Vector3D<f64, U>) -> Vector3D<f64, U> {
    rot_z(daily_phase, vec)
}

fn to_global_coords(axial_tilt: f64, axial_direction: f64, vec: Vector3D<f64, U>) -> Vector3D<f64, U> {
    rot_z(axial_direction, rot_y(axial_tilt, rot_z(-axial_direction, vec)))
}

fn get_normal_and_north(ts: f64) -> (Vector3D<f64, U>, Vector3D<f64, U>) {
    let daily_phase = get_phase(ts, INITIAL_DAILY_PHASE, SIDEREAL_DAY);
    let normal = to_global_coords(
        AXIAL_TILT,
        AXIAL_DIRECTION,
        to_recent_coords(daily_phase, to_local_coords(LAT, LON, vec3(1.0, 0.0, 0.0))),
    );
    let north = to_global_coords(
        AXIAL_TILT,
        AXIAL_DIRECTION,
        to_recent_coords(daily_phase, to_local_coords(LAT, LON, vec3(0.0, 0.0, 1.0))),
    );
    (normal, north)
}

fn get_sun_direction(phase: f64) -> Vector3D<f64, U> {
    -rot_z(phase, vec3(1.0, 0.0, 0.0))
}

fn get_moon_direction(moon_phase: f64) -> Vector3D<f64, U> {
    rot_z(moon_phase, vec3(1.0, 0.0, 0.0))
}

fn get_inclined_direction(to_moon: Vector3D<f64, U>, inclination: f64, nodal_phase: f64) -> Vector3D<f64, U> {
    _ = (inclination, nodal_phase);
    to_moon
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

impl Engine {
    pub fn new(ts: f64) -> Self {
        let (normal, north) = get_normal_and_north(ts);
        Self { ts, normal, north }
    }

    pub fn get_star_position(&self, star: &Star) -> (f64, f64) {
        let to_star = to_global_coords(
            AXIAL_TILT,
            AXIAL_DIRECTION,
            to_local_coords(star.declination, star.ascension, vec3(1.0, 0.0, 0.0)),
        );

        let alt = get_altitude(self.normal, to_star);
        let az = get_azimuth(self.normal, self.north, to_star);

        (alt, az)
    }

    pub fn get_sun_position(&self) -> (f64, f64) {
        let phase = get_phase(self.ts, INITIAL_PHASE, SIDEREAL);
        let to_sun = get_sun_direction(phase);

        let alt = get_altitude(self.normal, to_sun);
        let az = get_azimuth(self.normal, self.north, to_sun);

        (alt, az)
    }

    pub fn get_moon_position(&self) -> (f64, f64) {
        let moon_phase = get_phase(self.ts, INITIAL_MOON_PHASE, SIDEREAL_MONTH);
        let to_moon = get_moon_direction(moon_phase);

        let nodal_phase = get_phase(self.ts, INITIAL_NODAL_PHASE, NODAL_PERIOD);
        let to_moon = get_inclined_direction(to_moon, MOON_INCLINATION, nodal_phase);

        let alt = get_altitude(self.normal, to_moon);
        let az = get_azimuth(self.normal, self.north, to_moon);

        (alt, az)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_phase() {
        assert!((get_phase(0.0, INITIAL_PHASE, SIDEREAL) - INITIAL_PHASE).abs() < 1e-4);
        assert!(get_phase(22895580.0, INITIAL_PHASE, SIDEREAL).abs() < 1e-4);
        assert!(get_phase(811849260.0, INITIAL_PHASE, SIDEREAL).abs() < 1e-4);
        assert!((get_phase(1600802520.0, INITIAL_PHASE, SIDEREAL) - 2.0 * PI).abs() < 1e-4);
    }

    #[test]
    fn test_to_local_coords() {
        let normal = vec3(1.0, 0.0, 0.0);
        assert!((to_local_coords(0.0, 0.0, normal) - vec3(1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((to_local_coords(PI / 2.0, 0.0, normal) - vec3(0.0, 0.0, 1.0)).length() < 1e-15);
        assert!((to_local_coords(-PI / 2.0, 0.0, normal) - vec3(0.0, 0.0, -1.0)).length() < 1e-15);
        assert!((to_local_coords(0.0, PI / 2.0, normal) - vec3(0.0, 1.0, 0.0)).length() < 1e-15);
        assert!((to_local_coords(0.0, PI, normal) - vec3(-1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((to_local_coords(0.0, -PI / 2.0, normal) - vec3(0.0, -1.0, 0.0)).length() < 1e-15);
    }

    #[test]
    fn test_to_recent_coords() {
        let normal = vec3(1.0, 0.0, 0.0);
        assert!((to_recent_coords(0.0, normal) - vec3(1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((to_recent_coords(PI / 2.0, normal) - vec3(0.0, 1.0, 0.0)).length() < 1e-15);
    }

    #[test]
    fn test_to_global_coords() {
        let axis = vec3(0.0, 0.0, 1.0);
        assert!((to_global_coords(0.0, 0.0, axis) - vec3(0.0, 0.0, 1.0)).length() < 1e-15);
        assert!((to_global_coords(PI / 2.0, 0.0, axis) - vec3(1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((to_global_coords(PI / 2.0, PI / 2.0, axis) - vec3(0.0, 1.0, 0.0)).length() < 1e-15);
        assert!((to_global_coords(PI / 2.0, -PI / 2.0, axis) - vec3(0.0, -1.0, 0.0)).length() < 1e-15);
    }

    #[test]
    fn test_get_sun_direction() {
        assert!((get_sun_direction(0.0) - vec3(-1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((get_sun_direction(PI / 2.0) - vec3(0.0, -1.0, 0.0)).length() < 1e-15);
        assert!((get_sun_direction(PI) - vec3(1.0, 0.0, 0.0)).length() < 1e-15);
        assert!((get_sun_direction(3.0 * PI / 2.0) - vec3(0.0, 1.0, 0.0)).length() < 1e-15);
    }

    #[test]
    fn test_get_moon_direction() {
        assert!((get_moon_direction(PI / 2.0) - vec3(0.0, 1.0, 0.0)).length() < 1e-15);
    }

    #[test]
    fn test_get_altitude() {
        assert!((get_altitude(vec3(1.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0)) - PI / 2.0).abs() < 1e-15);
        assert!((get_altitude(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0)) - 0.0).abs() < 1e-15);
        assert!((get_altitude(vec3(1.0, 0.0, 0.0), vec3(-1.0, 0.0, 0.0)) + PI / 2.0).abs() < 1e-15);
    }

    #[test]
    fn test_get_azimuth() {
        assert!((get_azimuth(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.6, 0.8)) - 2.0 * PI).abs() < 1e-15);
        assert!((get_azimuth(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0)) - PI / 2.0).abs() < 1e-15);
    }
}
