import {readFileSync} from "fs";
import {join} from "path";

let ownerIdFile = ".ownerId";
let ownerId = readFileSync(join(__dirname, "..", ownerIdFile), "utf8");

export default ownerId;
