use std::f64::consts::PI;

use chrono::{DateTime, Utc};
use euclid::{vec3, Angle, Rotation3D, Vector3D};

use crate::{Planet, Star};

enum U {}

pub struct Engine {
    pub time: DateTime<Utc>,
    ts: f64,
    normal: Vector3D<f64, U>,
    north: Vector3D<f64, U>,
}

const INITIAL_PHASE: f64 = 1.740805; // sync with stellarium
const SIDEREAL_YEAR: f64 = 365.256363004 * 24.0 * 60.0 * 60.0; // stellarium
const SEMIMAJOR: f64 = 149.598; // nssdc.gsfc.nasa.gov

const INITIAL_DAILY_PHASE: f64 = 1.741395; // sync with stellarium
const SIDEREAL_DAY: f64 = 23.9344694 * 60.0 * 60.0; // stellarium

const AXIAL_TILT: f64 = 23.436169775089777 * PI / 180.0; // https://www.astro.sunysb.edu/fwalter/PHY515/coords.html
const AXIAL_DIRECTION: f64 = PI / 2.0; // https://www.astro.sunysb.edu/fwalter/PHY515/coords.html

const INITIAL_MOON_PHASE: f64 = 3.43; // eclipse
const SIDEREAL_MONTH: f64 = 27.321582 * 24.0 * 60.0 * 60.0; // stellarium

const MOON_INCLINATION: f64 = 5.145396 * PI / 180.0; // stellarium
const INITIAL_NODAL_PHASE: f64 = 5.0; // eclipse
const NODAL_PERIOD: f64 = 18.61 * SIDEREAL_YEAR;

const X_UNIT: Vector3D<f64, U> = vec3(1.0, 0.0, 0.0);
const Z_UNIT: Vector3D<f64, U> = vec3(0.0, 0.0, 1.0);

fn rot_y(angle: f64, vec: Vector3D<f64, U>) -> Vector3D<f64, U> {
    Rotation3D::around_y(Angle::radians(angle)).transform_vector3d(vec)
}

fn rot_z(angle: f64, vec: Vector3D<f64, U>) -> Vector3D<f64, U> {
    Rotation3D::around_z(Angle::radians(angle)).transform_vector3d(vec)
}

fn get_phase(ts: f64, initial_phase: f64, period: f64) -> f64 {
    (initial_phase + (ts / period * 2.0 * PI) % (2.0 * PI)) % (2.0 * PI)
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

fn get_normal_and_north(ts: f64, latitude: f64, longitude: f64) -> (Vector3D<f64, U>, Vector3D<f64, U>) {
    let daily_phase = get_phase(ts, INITIAL_DAILY_PHASE, SIDEREAL_DAY);
    let normal = to_global_coords(
        AXIAL_TILT,
        AXIAL_DIRECTION,
        to_recent_coords(daily_phase, to_local_coords(latitude, longitude, X_UNIT)),
    );
    let north = to_global_coords(
        AXIAL_TILT,
        AXIAL_DIRECTION,
        to_recent_coords(daily_phase, to_local_coords(latitude, longitude, Z_UNIT)),
    );
    (normal, north)
}

fn get_sun_direction(phase: f64) -> Vector3D<f64, U> {
    -rot_z(phase, X_UNIT)
}

fn get_object_direction(object_phase: f64) -> Vector3D<f64, U> {
    rot_z(object_phase, X_UNIT)
}

fn get_inclined_direction(to_moon: Vector3D<f64, U>, inclination: f64, nodal_phase: f64) -> Vector3D<f64, U> {
    rot_z(-nodal_phase, rot_y(-inclination, rot_z(nodal_phase, to_moon)))
}

fn get_altitude(normal: Vector3D<f64, U>, to_object: Vector3D<f64, U>) -> f64 {
    PI / 2.0 - normal.dot(to_object).acos()
}

fn get_azimuth(normal: Vector3D<f64, U>, north: Vector3D<f64, U>, to_object: Vector3D<f64, U>) -> f64 {
    let proj = (to_object - normal * normal.dot(to_object)).normalize();
    let angle = north.dot(proj).acos();
    let east = north.cross(normal);
    if east.dot(proj) > 0.0 {
        angle
    } else {
        2.0 * PI - angle
    }
}

fn get_lunar_phase(to_sun: Vector3D<f64, U>, to_moon: Vector3D<f64, U>) -> f64 {
    let angle = to_sun.dot(to_moon).acos();
    if to_sun.cross(to_moon).dot(Z_UNIT) > 0.0 {
        angle
    } else {
        2.0 * PI - angle
    }
}

fn get_moon_angle(
    normal: Vector3D<f64, U>,
    north: Vector3D<f64, U>,
    to_moon: Vector3D<f64, U>,
    to_sun: Vector3D<f64, U>,
    az: f64,
    lunar_phase: f64,
) -> f64 {
    let moon_to_sun = (to_sun - to_moon).normalize();
    let proj_moon_to_sun = (moon_to_sun - to_moon * to_moon.dot(moon_to_sun)).normalize();
    let east = north.cross(normal);
    let local_moon_to_sun = vec3(
        east.dot(proj_moon_to_sun),
        north.dot(proj_moon_to_sun),
        normal.dot(proj_moon_to_sun),
    );

    let level = rot_z(-az, X_UNIT);
    let angle = level.dot(local_moon_to_sun).acos();

    let local_to_moon = vec3(east.dot(to_moon), north.dot(to_moon), normal.dot(to_moon));
    let angle = if level.cross(local_moon_to_sun).dot(local_to_moon) > 0.0 {
        angle
    } else {
        2.0 * PI - angle
    };

    if lunar_phase < PI {
        (PI - az + angle) % (2.0 * PI)
    } else {
        (PI - az + angle + PI) % (2.0 * PI)
    }
}

impl Engine {
    pub fn new(time: DateTime<Utc>, latitude: f64, longitude: f64) -> Self {
        let ts = time.timestamp() as f64 + time.timestamp_subsec_nanos() as f64 * 1e-9;
        let (normal, north) = get_normal_and_north(ts, latitude, longitude);
        Self {
            time,
            ts,
            normal,
            north,
        }
    }

    pub fn get_star_position(&self, star: &Star) -> (f64, f64) {
        let to_star = to_global_coords(
            AXIAL_TILT,
            AXIAL_DIRECTION,
            to_local_coords(star.declination, star.ascension, X_UNIT),
        );

        let alt = get_altitude(self.normal, to_star);
        let az = get_azimuth(self.normal, self.north, to_star);

        (alt, az)
    }

    pub fn get_sun_position(&self) -> (f64, f64) {
        let phase = get_phase(self.ts, INITIAL_PHASE, SIDEREAL_YEAR);
        let to_sun = get_sun_direction(phase);

        let alt = get_altitude(self.normal, to_sun);
        let az = get_azimuth(self.normal, self.north, to_sun);

        (alt, az)
    }

    pub fn get_moon_position(&self) -> (f64, f64, f64, f64) {
        let moon_phase = get_phase(self.ts, INITIAL_MOON_PHASE, SIDEREAL_MONTH);
        let to_moon = get_object_direction(moon_phase);

        let nodal_phase = get_phase(self.ts, INITIAL_NODAL_PHASE, NODAL_PERIOD);
        let to_moon = get_inclined_direction(to_moon, MOON_INCLINATION, nodal_phase);

        let sun_phase = get_phase(self.ts, INITIAL_PHASE, SIDEREAL_YEAR);
        let to_sun = get_sun_direction(sun_phase);

        let alt = get_altitude(self.normal, to_moon);
        let az = get_azimuth(self.normal, self.north, to_moon);
        let lunar_phase = get_lunar_phase(to_sun, to_moon);
        let angle = get_moon_angle(self.normal, self.north, to_moon, to_sun, az, lunar_phase);

        (alt, az, lunar_phase, angle)
    }

    pub fn get_planet_position(&self, planet: &Planet) -> (f64, f64) {
        let phase = get_phase(self.ts, INITIAL_PHASE, SIDEREAL_YEAR);
        let to_earth = get_object_direction(phase);

        let phase = get_phase(self.ts, planet.phase, planet.sidereal);
        let to_planet = get_object_direction(phase);

        let earth_to_planet = (to_planet * planet.semimajor - to_earth * SEMIMAJOR).normalize();

        let alt = get_altitude(self.normal, earth_to_planet);
        let az = get_azimuth(self.normal, self.north, earth_to_planet);

        (alt, az)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const Y_UNIT: Vector3D<f64, U> = vec3(0.0, 1.0, 0.0);

    #[test]
    fn test_get_phase() {
        assert!((get_phase(0.0, INITIAL_PHASE, SIDEREAL_YEAR) - INITIAL_PHASE).abs() < 1e-4);
    }

    #[test]
    fn test_to_local_coords() {
        let normal = X_UNIT;
        assert!((to_local_coords(0.0, 0.0, normal) - X_UNIT).length() < 1e-15);
        assert!((to_local_coords(PI / 2.0, 0.0, normal) - Z_UNIT).length() < 1e-15);
        assert!((to_local_coords(-PI / 2.0, 0.0, normal) + Z_UNIT).length() < 1e-15);
        assert!((to_local_coords(0.0, PI / 2.0, normal) - Y_UNIT).length() < 1e-15);
        assert!((to_local_coords(0.0, PI, normal) + X_UNIT).length() < 1e-15);
        assert!((to_local_coords(0.0, -PI / 2.0, normal) + Y_UNIT).length() < 1e-15);
    }

    #[test]
    fn test_to_recent_coords() {
        let normal = X_UNIT;
        assert!((to_recent_coords(0.0, normal) - X_UNIT).length() < 1e-15);
        assert!((to_recent_coords(PI / 2.0, normal) - Y_UNIT).length() < 1e-15);
    }

    #[test]
    fn test_to_global_coords() {
        let axis = Z_UNIT;
        assert!((to_global_coords(0.0, 0.0, axis) - Z_UNIT).length() < 1e-15);
        assert!((to_global_coords(PI / 2.0, 0.0, axis) - X_UNIT).length() < 1e-15);
        assert!((to_global_coords(PI / 2.0, PI / 2.0, axis) - Y_UNIT).length() < 1e-15);
        assert!((to_global_coords(PI / 2.0, -PI / 2.0, axis) + Y_UNIT).length() < 1e-15);
    }

    #[test]
    fn test_get_sun_direction() {
        assert!((get_sun_direction(0.0) + X_UNIT).length() < 1e-15);
        assert!((get_sun_direction(PI / 2.0) + Y_UNIT).length() < 1e-15);
        assert!((get_sun_direction(PI) - X_UNIT).length() < 1e-15);
        assert!((get_sun_direction(3.0 * PI / 2.0) - Y_UNIT).length() < 1e-15);
    }

    #[test]
    fn test_get_object_direction() {
        assert!((get_object_direction(0.0) - X_UNIT).length() < 1e-15);
        assert!((get_object_direction(PI / 2.0) - Y_UNIT).length() < 1e-15);
    }

    #[test]
    fn test_get_inclined_direction() {
        assert!((get_inclined_direction(X_UNIT, PI / 2.0, 0.0) - Z_UNIT).length() < 1e-15);
        assert!((get_inclined_direction(Z_UNIT, PI / 2.0, 0.0) + X_UNIT).length() < 1e-15);
        assert!((get_inclined_direction(Z_UNIT, PI / 2.0, PI / 2.0) - Y_UNIT).length() < 1e-15);
    }

    #[test]
    fn test_get_altitude() {
        assert!((get_altitude(X_UNIT, X_UNIT) - PI / 2.0).abs() < 1e-15);
        assert!((get_altitude(X_UNIT, Y_UNIT) - 0.0).abs() < 1e-15);
        assert!((get_altitude(X_UNIT, -X_UNIT) + PI / 2.0).abs() < 1e-15);
    }

    #[test]
    fn test_get_azimuth() {
        assert!((get_azimuth(Z_UNIT, Y_UNIT, vec3(0.0, 0.6, 0.8)) - 2.0 * PI).abs() < 1e-15);
        assert!((get_azimuth(Z_UNIT, Y_UNIT, X_UNIT) - PI / 2.0).abs() < 1e-15);
    }

    #[test]
    fn test_get_lunar_phase() {
        assert!((get_lunar_phase(X_UNIT, X_UNIT) - 2.0 * PI).abs() < 1e-15);
        assert!((get_lunar_phase(X_UNIT, Y_UNIT) - PI / 2.0).abs() < 1e-15);
        assert!((get_lunar_phase(X_UNIT, -X_UNIT) - PI).abs() < 1e-15);
        assert!((get_lunar_phase(X_UNIT, -Y_UNIT) - 3.0 * PI / 2.0).abs() < 1e-15);
    }

    #[test]
    fn test_get_moon_angle() {
        let normal = Z_UNIT;
        let north = Y_UNIT;
        let to_moon = -X_UNIT;
        let to_sun = Y_UNIT;
        assert!(
            (get_moon_angle(normal, north, to_moon, to_sun, get_azimuth(normal, north, to_moon), 0.0) - 3.0 * PI / 2.0)
                .abs()
                < 1e-15
        );

        let normal = Z_UNIT;
        let north = Y_UNIT;
        let to_moon = Y_UNIT;
        let to_sun = X_UNIT;
        assert!(
            (get_moon_angle(normal, north, to_moon, to_sun, get_azimuth(normal, north, to_moon), 0.0) - PI).abs()
                < 1e-15
        );

        let normal = Z_UNIT;
        let north = Y_UNIT;
        let to_moon = -X_UNIT;
        let to_sun = Z_UNIT;
        assert!(
            (get_moon_angle(normal, north, to_moon, to_sun, get_azimuth(normal, north, to_moon), 0.0) - PI).abs()
                < 1e-15
        );

        let normal = Z_UNIT;
        let north = Y_UNIT;
        let to_moon = -X_UNIT;
        let to_sun = Y_UNIT;
        assert!(
            (get_moon_angle(normal, north, to_moon, to_sun, get_azimuth(normal, north, to_moon), 4.0) - PI / 2.0).abs()
                < 1e-15
        );
    }
}
