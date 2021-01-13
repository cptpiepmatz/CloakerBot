"use strict";

const fs = require("fs");
const path = require("path");

const args = process.argv.slice(2);
let tokenFile = ".token";
if (args.includes("test")) {
  tokenFile = "test.token";
}
let token = fs.readFileSync(path.join(__dirname, "..", tokenFile), "utf8");

module.exports = token;