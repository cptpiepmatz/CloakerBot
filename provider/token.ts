import {join} from "path";

import FileProviderUtil from "./FileProviderUtil";

const args = process.argv.slice(2);
const tokenFile = args.includes("test") ? "test.token" : ".token";
const tokenFilePath = join(__dirname, "..", tokenFile);

export default FileProviderUtil.provideString(tokenFilePath);
