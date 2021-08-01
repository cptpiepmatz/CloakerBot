import {join} from "path";

import FileProviderUtil from "./FileProviderUtil";

const repliesPath = join(__dirname, "../assets/replies.json5");

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

export default FileProviderUtil.provideObject<Replies>(repliesPath);
