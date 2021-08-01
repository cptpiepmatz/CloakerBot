import {join} from "path";

import FileProviderUtil from "./FileProviderUtil";

const ownerIdFile = ".ownerId";
const ownerIdFilePath = join(__dirname, "..", ownerIdFile);

export default FileProviderUtil.provideString(ownerIdFilePath);
