"use strict";

const fs = require("fs");
const path = require("path");

let ownerIDFile = ".ownerId";
let ownerID = fs.readFileSync(
  path.join(__dirname, "..", ownerIDFile), "utf8");

module.exports = ownerID;
