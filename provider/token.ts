import {readFileSync} from "fs";
import {join} from "path";

const args = process.argv.slice(2);
const tokenFile = args.includes("test") ? "test.token" : ".token";
const token = readFileSync(join(__dirname, "..", tokenFile), "utf8");

export default token;
