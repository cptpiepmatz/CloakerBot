import {readdirSync, readFileSync} from "fs";
import {join} from "path";

/** Simple recursive type for buffers I use for the audio files. */
type AudioBuffers = {
  [key: string]: (Buffer | AudioBuffers)
}

const soundsDir = join(__dirname, "../assets/sounds/generated");

/**
 * Reads recursively files into a single object using buffers.
 * @param directory The directory to further search in
 */
function readIntoBuffers(directory: string = "") {
  let audioBuffers: AudioBuffers = {};
  let dirents = readdirSync(join(soundsDir, directory), {withFileTypes: true});
  for (let dirent of dirents) {
    if (dirent.isFile()) {
      audioBuffers[dirent.name.replace(".ogg", "")]
        = readFileSync(join(soundsDir, directory, dirent.name));
      continue;
    }
    if (dirent.isDirectory()) {
      audioBuffers["dir:" + dirent.name]
        = readIntoBuffers(join(directory, dirent.name));
    }
  }
  return audioBuffers;
}

export default readIntoBuffers();
