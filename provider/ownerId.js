"use strict";

const fs = require("fs");
const path = require("path");

let ownerIDFile = ".ownerId";
let ownerId = fs.readFileSync(
  path.join(__dirname, "..", ownerIDFile), "utf8");

module.exports = ownerId;
