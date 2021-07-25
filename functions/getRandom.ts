/**
 * Returns a random element from an array.
 * @param array
 */
export default function getRandom<T>(array: T[]): T {
  let index = Math.floor(Math.random() * array.length);
  return array[index];
}
