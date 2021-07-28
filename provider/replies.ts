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
  doRandom: string[];
  doNotRandom: string[];
  incorrectInput: string[];
  takedownSuccessful: string[];
  unknownInput: string[];
  noPerms: string[];
  noMember: string[];
  noChannel: string[];
  cantJoin: string[];
  failedTakedown: string[];
  noNumber: string[];
}

export default repliesObject as Replies;
