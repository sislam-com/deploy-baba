function minMovesToMakeComplementary(nums: number[], limit: number): number {
    let moves = 0;
    const n = nums.length;
    
    for (let i = 0; i < n / 2; i++) {
        const j = n - 1 - i;
        const a = nums[i];
        const b = nums[j];
        
        // Check if both elements are already valid (≤ limit)
        const aValid = a <= limit;
        const bValid = b <= limit;
        
        // Check if pair sums to limit
        if (a + b === limit && aValid && bValid) {
            // Already complementary, no moves needed
            continue;
        }
        
        // Need to fix this pair
        if (aValid && bValid) {
            // Both valid but wrong sum - change one element
            moves++;
        } else if (aValid || bValid) {
            // One valid, one invalid - change the invalid one
            moves++;
        } else {
            // Both invalid - need to change both
            moves += 2;
        }
    }
    
    return moves;
}

// Test with all examples
console.log("Example 1:");
const nums1 = [1, 2, 4, 3];
const limit1 = 4;
const result1 = minMovesToMakeComplementary(nums1, limit1);
console.log(`Input: nums = [${nums1}], limit = ${limit1}`);
console.log(`Output: ${result1}`);
console.log(`Expected: 1\n`);

console.log("Example 2:");
const nums2 = [1, 2, 2, 1];
const limit2 = 2;
const result2 = minMovesToMakeComplementary(nums2, limit2);
console.log(`Input: nums = [${nums2}], limit = ${limit2}`);
console.log(`Output: ${result2}`);
console.log(`Expected: 2\n`);

console.log("Example 3:");
const nums3 = [1, 2, 1, 2];
const limit3 = 2;
const result3 = minMovesToMakeComplementary(nums3, limit3);
console.log(`Input: nums = [${nums3}], limit = ${limit3}`);
console.log(`Output: ${result3}`);
console.log(`Expected: 0`);
