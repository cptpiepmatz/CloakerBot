"use strict";

const fs = require("fs");
const path = require("path");
const json5 = require("json5");

let repliesPath = path.join(__dirname, "..", "replies.json5");
let repliesFile = fs.readFileSync(repliesPath, "utf8");
let repliesObject = json5.parse(repliesFile);

/**
 * @type {Replies}
 */
module.exports = repliesObject;

/**
 * Reply object containing all reply string
 *
 * @typedef {Object} Replies
 * @property {string[]} doRandom
 * @property {string[]} doNotRandom
 * @property {string[]} incorrectInput
 * @property {string[]} unknownInput
 * @property {string[]} noPerms
 * @property {string[]} noMember
 * @property {string[]} noChannel
 * @property {string[]} cantJoin
 * @property {string[]} failedTakedown
 * @property {string[]} noNumber
 **/