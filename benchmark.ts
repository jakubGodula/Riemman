import { Complex } from './src/lib/math/complex';
import { partialSumSequence } from './src/lib/math/zeta';

const s = new Complex(0.5, 14.1347);
const N = 1000000;

console.time('zeta_1m');
const seq = partialSumSequence(s, N);
console.timeEnd('zeta_1m');
console.log('Final sum:', seq[N-1]);
