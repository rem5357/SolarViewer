// Motion models and orbital mechanics for StellarForge

use crate::stellar_forge::core::{State, Vec3, Units};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// Main motion model enum
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MotionModel {
    Keplerian(OrbitalElements),
    Free(FreeMotion),
    TableEphemeris(EphemerisTable),
    Scripted(ScriptedMotion),
    TwoBody(TwoBodyMotion),
    NBody(NBodyMotion),
}

impl MotionModel {
    // Propagate state from one epoch to another
    pub fn propagate(&self, initial: State, from: OffsetDateTime, to: OffsetDateTime) -> State {
        let dt = (to - from).as_seconds_f64();

        match self {
            MotionModel::Keplerian(elements) => elements.propagate(initial, dt),
            MotionModel::Free(free) => free.propagate(initial, dt),
            MotionModel::TableEphemeris(table) => table.interpolate(to),
            MotionModel::Scripted(scripted) => scripted.evaluate(to),
            MotionModel::TwoBody(twobody) => twobody.propagate(initial, dt),
            MotionModel::NBody(nbody) => nbody.propagate(initial, dt),
        }
    }

    pub fn orbital_period_s(&self) -> Option<f64> {
        match self {
            MotionModel::Keplerian(e) => Some(e.orbital_period()),
            MotionModel::TwoBody(tb) => Some(tb.orbital_period()),
            _ => None,
        }
    }
}

// Keplerian orbital elements
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct OrbitalElements {
    pub semi_major_axis_m: f64,      // a
    pub eccentricity: f64,            // e
    pub inclination_rad: f64,         // i
    pub longitude_ascending_rad: f64, // Ω (RAAN)
    pub argument_periapsis_rad: f64,  // ω
    pub mean_anomaly_rad: f64,        // M at epoch
    pub gravitational_param_m3s2: f64, // μ = GM
    pub epoch: OffsetDateTime,
}

impl OrbitalElements {
    pub fn new(
        a: f64,
        e: f64,
        i: f64,
        omega: f64,
        w: f64,
        m0: f64,
        mu: f64,
        epoch: OffsetDateTime,
    ) -> Self {
        Self {
            semi_major_axis_m: a,
            eccentricity: e,
            inclination_rad: i,
            longitude_ascending_rad: omega,
            argument_periapsis_rad: w,
            mean_anomaly_rad: m0,
            gravitational_param_m3s2: mu,
            epoch,
        }
    }

    // Create circular orbit
    pub fn circular(radius: f64, mu: f64, epoch: OffsetDateTime) -> Self {
        Self {
            semi_major_axis_m: radius,
            eccentricity: 0.0,
            inclination_rad: 0.0,
            longitude_ascending_rad: 0.0,
            argument_periapsis_rad: 0.0,
            mean_anomaly_rad: 0.0,
            gravitational_param_m3s2: mu,
            epoch,
        }
    }

    // Create from state vectors
    pub fn from_state_vectors(state: State, mu: f64, epoch: OffsetDateTime) -> Result<Self, String> {
        let r = state.position_m;
        let v = state.velocity_mps;
        let r_mag = r.norm();
        let v_mag = v.norm();

        if r_mag == 0.0 {
            return Err("Position vector is zero".into());
        }

        // Angular momentum
        let h = r.cross(&v);
        let h_mag = h.norm();

        if h_mag == 0.0 {
            return Err("Angular momentum is zero (rectilinear motion)".into());
        }

        // Eccentricity vector
        let e_vec = ((v.cross(&h)) / mu) - (r / r_mag);
        let e = e_vec.norm();

        // Semi-major axis
        let a = if e != 1.0 {
            1.0 / ((2.0 / r_mag) - (v_mag * v_mag / mu))
        } else {
            return Err("Parabolic orbit (e = 1)".into());
        };

        // Inclination
        let i = (h.z / h_mag).acos();

        // Node vector
        let n = Vec3::new(0.0, 0.0, 1.0).cross(&h);
        let n_mag = n.norm();

        // Longitude of ascending node
        let omega = if n_mag != 0.0 {
            let omega = (n.x / n_mag).acos();
            if n.y < 0.0 {
                2.0 * std::f64::consts::PI - omega
            } else {
                omega
            }
        } else {
            0.0
        };

        // Argument of periapsis
        let w = if n_mag != 0.0 && e > 0.0 {
            let w = (n.dot(&e_vec) / (n_mag * e)).acos();
            if e_vec.z < 0.0 {
                2.0 * std::f64::consts::PI - w
            } else {
                w
            }
        } else {
            0.0
        };

        // True anomaly
        let nu = if e > 0.0 {
            let nu = (e_vec.dot(&r) / (e * r_mag)).acos();
            if r.dot(&v) < 0.0 {
                2.0 * std::f64::consts::PI - nu
            } else {
                nu
            }
        } else {
            // Circular orbit: use angle from node
            if n_mag > 0.0 {
                let nu = (n.dot(&r) / (n_mag * r_mag)).acos();
                if r.z < 0.0 {
                    2.0 * std::f64::consts::PI - nu
                } else {
                    nu
                }
            } else {
                // Equatorial circular orbit
                r.y.atan2(r.x)
            }
        };

        // Mean anomaly from true anomaly
        let ea = Self::true_to_eccentric_anomaly(nu, e);
        let m = Self::eccentric_to_mean_anomaly(ea, e);

        Ok(Self {
            semi_major_axis_m: a,
            eccentricity: e,
            inclination_rad: i,
            longitude_ascending_rad: omega,
            argument_periapsis_rad: w,
            mean_anomaly_rad: m,
            gravitational_param_m3s2: mu,
            epoch,
        })
    }

    // Propagate orbital elements by time dt (seconds)
    pub fn propagate(&self, _initial: State, dt: f64) -> State {
        // Mean motion
        let n = (self.gravitational_param_m3s2 / (self.semi_major_axis_m.powi(3))).sqrt();

        // Mean anomaly at time t
        let m = self.mean_anomaly_rad + n * dt;

        // Solve Kepler's equation for eccentric anomaly
        let e_anom = self.solve_kepler(m, self.eccentricity);

        // True anomaly
        let nu = self.eccentric_to_true_anomaly(e_anom, self.eccentricity);

        // Position in orbital plane
        let r = self.semi_major_axis_m * (1.0 - self.eccentricity * self.eccentricity)
            / (1.0 + self.eccentricity * nu.cos());

        let pos_orbital = Vec3::new(r * nu.cos(), r * nu.sin(), 0.0);

        // Velocity in orbital plane
        let h = (self.gravitational_param_m3s2 * self.semi_major_axis_m
            * (1.0 - self.eccentricity * self.eccentricity)).sqrt();

        let vr = h * self.eccentricity * nu.sin()
            / (self.semi_major_axis_m * (1.0 - self.eccentricity * self.eccentricity));
        let vtheta = h / r;

        let vel_orbital = Vec3::new(
            vr * nu.cos() - vtheta * nu.sin(),
            vr * nu.sin() + vtheta * nu.cos(),
            0.0,
        );

        // Transform to inertial frame
        let (pos_inertial, vel_inertial) = self.orbital_to_inertial(pos_orbital, vel_orbital);

        State {
            position_m: pos_inertial,
            velocity_mps: vel_inertial,
        }
    }

    // Transform from orbital plane to inertial coordinates
    fn orbital_to_inertial(&self, r_orbital: Vec3, v_orbital: Vec3) -> (Vec3, Vec3) {
        let cos_omega = self.longitude_ascending_rad.cos();
        let sin_omega = self.longitude_ascending_rad.sin();
        let cos_i = self.inclination_rad.cos();
        let sin_i = self.inclination_rad.sin();
        let cos_w = self.argument_periapsis_rad.cos();
        let sin_w = self.argument_periapsis_rad.sin();

        // Rotation matrix from orbital to inertial
        let r11 = cos_omega * cos_w - sin_omega * sin_w * cos_i;
        let r12 = -cos_omega * sin_w - sin_omega * cos_w * cos_i;
        let r13 = sin_omega * sin_i;

        let r21 = sin_omega * cos_w + cos_omega * sin_w * cos_i;
        let r22 = -sin_omega * sin_w + cos_omega * cos_w * cos_i;
        let r23 = -cos_omega * sin_i;

        let r31 = sin_w * sin_i;
        let r32 = cos_w * sin_i;
        let r33 = cos_i;

        // Apply rotation
        let r_inertial = Vec3::new(
            r11 * r_orbital.x + r12 * r_orbital.y + r13 * r_orbital.z,
            r21 * r_orbital.x + r22 * r_orbital.y + r23 * r_orbital.z,
            r31 * r_orbital.x + r32 * r_orbital.y + r33 * r_orbital.z,
        );

        let v_inertial = Vec3::new(
            r11 * v_orbital.x + r12 * v_orbital.y + r13 * v_orbital.z,
            r21 * v_orbital.x + r22 * v_orbital.y + r23 * v_orbital.z,
            r31 * v_orbital.x + r32 * v_orbital.y + r33 * v_orbital.z,
        );

        (r_inertial, v_inertial)
    }

    // Solve Kepler's equation: M = E - e*sin(E)
    fn solve_kepler(&self, m: f64, e: f64) -> f64 {
        let tolerance = 1e-10;
        let max_iterations = 100;

        // Initial guess
        let mut e_anom = if e < 0.8 { m } else { std::f64::consts::PI };

        for _ in 0..max_iterations {
            let f = e_anom - e * e_anom.sin() - m;
            let df = 1.0 - e * e_anom.cos();

            let de = f / df;
            e_anom -= de;

            if de.abs() < tolerance {
                break;
            }
        }

        e_anom
    }

    // Convert eccentric anomaly to true anomaly
    fn eccentric_to_true_anomaly(&self, e_anom: f64, e: f64) -> f64 {
        let beta = e / (1.0 + (1.0 - e * e).sqrt());
        let nu = e_anom + 2.0 * (beta * e_anom.sin() / (1.0 - beta * e_anom.cos())).atan();
        nu
    }

    // Convert true anomaly to eccentric anomaly
    fn true_to_eccentric_anomaly(nu: f64, e: f64) -> f64 {
        ((1.0 - e).sqrt() * (nu / 2.0).tan()).atan() * 2.0
    }

    // Convert eccentric anomaly to mean anomaly
    fn eccentric_to_mean_anomaly(e_anom: f64, e: f64) -> f64 {
        e_anom - e * e_anom.sin()
    }

    // Calculate orbital period
    pub fn orbital_period(&self) -> f64 {
        2.0 * std::f64::consts::PI
            * (self.semi_major_axis_m.powi(3) / self.gravitational_param_m3s2).sqrt()
    }

    // Calculate mean motion (rad/s)
    pub fn mean_motion(&self) -> f64 {
        (self.gravitational_param_m3s2 / self.semi_major_axis_m.powi(3)).sqrt()
    }

    // Get apoapsis distance
    pub fn apoapsis(&self) -> f64 {
        self.semi_major_axis_m * (1.0 + self.eccentricity)
    }

    // Get periapsis distance
    pub fn periapsis(&self) -> f64 {
        self.semi_major_axis_m * (1.0 - self.eccentricity)
    }

    // Check if orbit is hyperbolic
    pub fn is_hyperbolic(&self) -> bool {
        self.eccentricity >= 1.0
    }

    // Check if orbit is retrograde
    pub fn is_retrograde(&self) -> bool {
        self.inclination_rad > std::f64::consts::PI / 2.0
    }
}

// Free motion (constant velocity)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FreeMotion {
    pub initial_state: State,
    pub epoch: OffsetDateTime,
    pub acceleration_mps2: Option<Vec3>,  // Optional constant acceleration
}

impl FreeMotion {
    pub fn new(state: State, epoch: OffsetDateTime) -> Self {
        Self {
            initial_state: state,
            epoch,
            acceleration_mps2: None,
        }
    }

    pub fn with_acceleration(state: State, epoch: OffsetDateTime, accel: Vec3) -> Self {
        Self {
            initial_state: state,
            epoch,
            acceleration_mps2: Some(accel),
        }
    }

    pub fn propagate(&self, _initial: State, dt: f64) -> State {
        let mut pos = self.initial_state.position_m + self.initial_state.velocity_mps * dt;
        let mut vel = self.initial_state.velocity_mps;

        if let Some(accel) = self.acceleration_mps2 {
            vel += accel * dt;
            pos += accel * 0.5 * dt * dt;
        }

        State {
            position_m: pos,
            velocity_mps: vel,
        }
    }
}

// Ephemeris table for high-precision propagation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EphemerisTable {
    pub samples: Vec<EphemerisSample>,
    pub interpolation: InterpolationMethod,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EphemerisSample {
    pub epoch: OffsetDateTime,
    pub state: State,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum InterpolationMethod {
    Linear,
    CubicSpline,
    Hermite,
    Lagrange(usize),  // Order
}

impl EphemerisTable {
    pub fn new(samples: Vec<EphemerisSample>) -> Self {
        Self {
            samples,
            interpolation: InterpolationMethod::CubicSpline,
        }
    }

    pub fn interpolate(&self, epoch: OffsetDateTime) -> State {
        // Find bracketing samples
        let idx = self.samples
            .binary_search_by_key(&epoch, |s| s.epoch)
            .unwrap_or_else(|i| i.saturating_sub(1).min(self.samples.len() - 2));

        if idx >= self.samples.len() - 1 {
            return self.samples.last().unwrap().state;
        }

        let s0 = &self.samples[idx];
        let s1 = &self.samples[idx + 1];

        // Linear interpolation (simplified)
        let dt_total = (s1.epoch - s0.epoch).as_seconds_f64();
        let dt = (epoch - s0.epoch).as_seconds_f64();
        let t = dt / dt_total;

        State {
            position_m: s0.state.position_m * (1.0 - t) + s1.state.position_m * t,
            velocity_mps: s0.state.velocity_mps * (1.0 - t) + s1.state.velocity_mps * t,
        }
    }
}

// Scripted motion for custom behaviors
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptedMotion {
    pub script_id: String,
    pub parameters: serde_json::Value,
    pub epoch: OffsetDateTime,
}

impl ScriptedMotion {
    pub fn evaluate(&self, epoch: OffsetDateTime) -> State {
        // This would call out to a scripting system
        // For now, return a default state
        State {
            position_m: Vec3::zeros(),
            velocity_mps: Vec3::zeros(),
        }
    }
}

// Two-body problem with perturbations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TwoBodyMotion {
    pub primary_mass_kg: f64,
    pub secondary_mass_kg: f64,
    pub elements: OrbitalElements,
    pub perturbations: Vec<Perturbation>,
}

impl TwoBodyMotion {
    pub fn orbital_period(&self) -> f64 {
        self.elements.orbital_period()
    }

    pub fn propagate(&self, initial: State, dt: f64) -> State {
        // Start with Keplerian propagation
        let mut state = self.elements.propagate(initial, dt);

        // Apply perturbations
        for perturbation in &self.perturbations {
            state = perturbation.apply(state, dt);
        }

        state
    }
}

// Perturbation forces
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Perturbation {
    J2 { coefficient: f64, body_radius_m: f64 },
    SolarRadiationPressure { area_m2: f64, mass_kg: f64, cr: f64 },
    AtmosphericDrag { cd: f64, area_m2: f64, mass_kg: f64 },
    ThirdBody { mass_kg: f64, position: Vec3 },
}

impl Perturbation {
    pub fn apply(&self, state: State, _dt: f64) -> State {
        // Simplified perturbation application
        // In reality, this would compute accelerations and integrate
        state
    }
}

// N-body motion
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NBodyMotion {
    pub bodies: Vec<NBodyMass>,
    pub integrator: IntegratorType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NBodyMass {
    pub mass_kg: f64,
    pub position_m: Vec3,
    pub velocity_mps: Vec3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum IntegratorType {
    Euler,
    RungeKutta4,
    Verlet,
    LeapFrog,
    AdaptiveRK45,
}

impl NBodyMotion {
    pub fn propagate(&self, _initial: State, _dt: f64) -> State {
        // N-body integration would go here
        // This is computationally intensive and would use the selected integrator
        State {
            position_m: Vec3::zeros(),
            velocity_mps: Vec3::zeros(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_orbit() {
        let mu = 3.986e14;  // Earth's gravitational parameter
        let radius = 7e6;   // 7000 km
        let epoch = OffsetDateTime::now_utc();

        let elements = OrbitalElements::circular(radius, mu, epoch);

        assert_eq!(elements.eccentricity, 0.0);
        assert_eq!(elements.semi_major_axis_m, radius);

        let period = elements.orbital_period();
        assert!((period - 5800.0).abs() < 100.0);  // ~96 minutes
    }

    #[test]
    fn test_kepler_solver() {
        let epoch = OffsetDateTime::now_utc();
        let elements = OrbitalElements::new(
            7e6,  // 7000 km
            0.1,  // e = 0.1
            0.0,  // i = 0
            0.0,  // Ω = 0
            0.0,  // ω = 0
            0.0,  // M0 = 0
            3.986e14,  // Earth μ
            epoch,
        );

        let e_anom = elements.solve_kepler(0.5, 0.1);
        let m_check = elements.eccentric_to_mean_anomaly(e_anom, 0.1);

        assert!((m_check - 0.5).abs() < 1e-9);
    }
}