"use strict";

/**
 * Returns a random element from an array.
 * @param {*[]} array
 * @returns {*}
 */
function getRandom(array) {
  let index = Math.floor(Math.random() * array.length);
  return array[index];
}

module.exports = getRandom;