/**
 * Checks if the passed arguments contains null.
 * If it contains null the function returns null.
 * Otherwise it just runs the function.
 * @param passFunc Function to execute with the given parameters
 * @param passParams Parameters to test for null or execute
 */
const runOrNull = <T>(
  passFunc: (...internalParams: any) => T,
  ...passParams: any
): T | null  => {
  if (passParams.includes(null)) return null;
  return passFunc(...passParams);
}

export default runOrNull;
