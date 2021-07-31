use std::time::{Duration, Instant};

use quinn_proto::congestion::{Controller, ControllerFactory};

#[derive(Clone, Debug)]
pub struct FixedRateConfig {
    pub kbits_per_sec: u64,
}

#[derive(Clone)]
pub struct FixedRate {
    config: FixedRateConfig,
    rtt_msec_sum: u64,
    rtt_msec_weight: u64,
    window: u64,
}

impl Controller for FixedRate {
    fn on_ack(
        &mut self,
        _now: Instant,
        _sent: Instant,
        _bytes: u64,
        _app_limited: bool,
        rtt: Duration,
    ) {
        // Exponential moving average of RTT time
        self.rtt_msec_sum -= self.rtt_msec_sum >> 8;
        self.rtt_msec_weight -= self.rtt_msec_weight >> 8;
        self.rtt_msec_sum += (rtt.as_millis() as u64) << 32;
        self.rtt_msec_weight += 1 << 32;

        let rtt_msec = self.rtt_msec_sum / self.rtt_msec_weight;

        // Compute window that would reach the target rate
        self.window = self.config.kbits_per_sec * rtt_msec / 8; // bits -> bytes
    }

    fn on_congestion_event(
        &mut self,
        _now: Instant,
        _sent: Instant,
        _is_persistent_congestion: bool,
    ) {
        // Ignore congestion events
    }

    fn window(&self) -> u64 {
        self.window
    }

    fn clone_box(&self) -> Box<dyn Controller> {
        Box::new(self.clone())
    }

    fn initial_window(&self) -> u64 {
        self.config.kbits_per_sec * (1000 / 8)
    }
}

impl FixedRate {
    pub fn new(config: FixedRateConfig) -> Self {
        Self {
            rtt_msec_sum: 1000,
            rtt_msec_weight: 1,
            window: config.kbits_per_sec * (1000 / 8),
            config,
        }
    }
}

impl ControllerFactory for FixedRateConfig {
    fn build(&self, _now: Instant) -> Box<dyn Controller> {
        Box::new(FixedRate::new(self.clone()))
    }
}
