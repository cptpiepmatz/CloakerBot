import {readFileSync} from "fs";
import {parse} from "json5";

/** Utility class to provide files. */
export default class FileProviderUtil {

  /** Provides the raw string. */
  static provideString(filePath: string): string {
    return readFileSync(filePath, "utf8");
  }

  /** Provide a parsed object. Also capable to work with json5. */
  static provideObject<TProvide>(filePath: string): TProvide {
    const fileContent = readFileSync(filePath, "utf8");
    if (filePath.endsWith(".json5")) return parse(fileContent);
    return JSON.parse(fileContent);
  }

}
