import {EmbedFieldData} from "discord.js";
import {join} from "path";

import FileProviderUtil from "./FileProviderUtil";

const helpTextPath = join(__dirname, "../assets/help.json5");

/** Interface describing how the help text object is defined. */
export interface HelpText {
  title: string;
  description: string;
  fields: EmbedFieldData[];
}

export default FileProviderUtil.provideObject<HelpText>(helpTextPath);
