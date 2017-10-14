pub struct EnglishCharScore<T> {
    best_score: u64,
    best: Option<T>,
    total: u64,
    freqs: [u64; 256],

}

impl<T> EnglishCharScore<T> {
    pub fn new() -> EnglishCharScore<T> {
        EnglishCharScore {
            best_score: u64::max_value(),
            best: None,
            total: 0,
            freqs: [0u64; 256],
        }
    }

    fn reset(&mut self) {
        for i in self.freqs.iter_mut() {
            *i = 0;
        }
        self.total = 0;
    }

    pub fn add_byte(&mut self, b: u8) {
        self.freqs[b as usize] += 1;
        self.total += 1;
    }
    
    pub fn update_best(&mut self, val: T) {
        // TODO: maybe should collapse lower and upper case values
        let score = english_freq_score(&self.freqs, self.total);
        if score < self.best_score {
            // XXX: same score?
            self.best_score = score;
            self.best = Some(val);
        }
        self.reset();
    }

    pub fn get_best(&self) -> (u64, &Option<T>) {
        (self.best_score, &self.best)
    }
}

const ENGLISH_BYTE_SCALE: u64 = 10000;
const ENGLISH_BYTE_FREQS: [u64; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1076, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 724, 132, 241, 385, 1072, 205, 180, 528, 651, 9, 61, 355, 233, 619, 685,
    162, 10, 537, 560, 811, 256, 98, 186, 15, 188, 6, 0, 0, 0, 0, 0,
    0, 724, 132, 241, 385, 1072, 205, 180, 528, 651, 9, 61, 355, 233, 619, 685,
    162, 10, 537, 560, 811, 256, 98, 186, 15, 188, 6, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub fn english_freq_score(freqs: &[u64; 256], total: u64) -> u64 {
    let mut score: u64 = 0;
    for (&freq, &expected_freq) in
        freqs.iter().zip(ENGLISH_BYTE_FREQS.iter()) {
        let normalized_freq = (freq * ENGLISH_BYTE_SCALE) / total;
        let mut diff = (expected_freq as i64) - (normalized_freq as i64);
        if expected_freq == 0 && freq > 0 {
            // unexpected characters like ^ should be penalized more probably
            // this is kind of a hack for now though
            diff *= 2;
        }

        // chi squared doesn't behave well if expected values are 0
        // could do fisher's exact test or barnard's exact test but seems
        // overkill so just do simple diff addition
        score += diff.abs() as u64;
    }
    score
}
