import {readFileSync} from "fs";
import {join} from "path";
import {parse} from "json5";

const repliesPath = join(__dirname, "../assets/replies.json5");
const repliesFile = readFileSync(repliesPath, "utf8");
const repliesObject = parse(repliesFile);

/**
 * Reply interface containing all reply strings.
 */
interface Replies {
  doRandom: String[];
  doNotRandom: String[];
  incorrectInput: String[];
  unknownInput: String[];
  noPerms: String[];
  noMember: String[];
  noChannel: String[];
  cantJoin: String[];
  failedTakedown: String[];
  noNumber: String[];
}

export default repliesObject as Replies;
