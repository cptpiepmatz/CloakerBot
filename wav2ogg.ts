import ffmpeg from "ffmpeg-static";
import fs from "fs/promises";
import {Presets, SingleBar} from "cli-progress";
import {exec} from "child_process";
import {join} from "path";

const soundsDir = join(__dirname, "assets/sounds");
const failsDir = join(soundsDir, "fails");
const takedownsDir = join(soundsDir, "takedowns");
const outputDir = join(soundsDir, "generated");
const outputFailsDir = join(outputDir, "fails");
const outputTakedownsDir = join(outputDir, "takedowns");

/**
 * Subroutine to read in wav file paths.
 * @param dir Directory to search in
 * @return Array of wav file paths
 */
async function readWavFiles(dir: string): Promise<string[]> {
  let wavFiles: string[] = []
  for (let dirent of await fs.readdir(dir, {withFileTypes: true})) {
    if (dirent.isFile() && dirent.name.endsWith(".wav")) {
      wavFiles.push(dirent.name);
    }
  }
  return wavFiles;
}

/**
 * Executes ffmpeg to convert the .wav files to .ogg.
 * @param filePath Path of .wav file
 * @param outputPath Path of generated .ogg file
 * @param volume Volume modifier
 */
function convertFile(
  filePath: string,
  outputPath: string,
  volume: string = "1.0"
): Promise<void> {
  return new Promise((resolve, reject) => {
    let command = `"${ffmpeg}" -i "${filePath}" -c:a libopus -b:a 96k -filter:a "volume=${volume}" "${outputPath}"`;
    exec(
      command,
      (error) => {
        if (error) reject(error);
        resolve();
      }
    );
  });
}

(async function() {
  const rootFiles = await readWavFiles(soundsDir);
  const failFiles = await readWavFiles(failsDir);
  const takedownFiles = await readWavFiles(takedownsDir);

  console.log("Removing old files...");
  await fs.rm(outputDir, {recursive: true, force: true});

  console.log("Generating directory structure...");
  await fs.mkdir(outputDir);
  await fs.mkdir(outputFailsDir);
  await fs.mkdir(outputTakedownsDir);

  console.log("Converting .wav files to .ogg...");
  const progressBar = new SingleBar(Presets.shades_classic);
  progressBar.start(
    rootFiles.length + failFiles.length + takedownFiles.length, 0);

  try {
    // TODO: check for volume values
    for (let rootFile of rootFiles) {
      await convertFile(
        join(soundsDir, rootFile),
        join(outputDir, rootFile).replace(/.wav$/, ".ogg"));
      progressBar.increment(1);
    }

    for (let failFile of failFiles) {
      await convertFile(
        join(failsDir, failFile),
        join(outputFailsDir, failFile).replace(/.wav$/, ".ogg")
      );
      progressBar.increment(1);
    }

    for (let takedownFile of takedownFiles) {
      await convertFile(
        join(takedownsDir, takedownFile),
        join(outputTakedownsDir, takedownFile).replace(/.wav$/, ".ogg")
      );
      progressBar.increment(1);
    }
  }
  catch (error) {
    console.log();
    if (error instanceof Error) {
      let splittenError = error.message.split("\n");
      console.error(splittenError[splittenError.length - 2]);
    }
    else {
      console.error(error);
    }
    process.exit(1);
  }

  progressBar.stop();
})();
