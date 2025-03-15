fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

pub fn lcm(nums: &[usize]) -> usize {
    if nums.len() == 1 {
        nums[0]
    } else {
        let a = nums[0];
        let b = lcm(&nums[1..]);
        a * b / gcd(a, b)
    }
}
