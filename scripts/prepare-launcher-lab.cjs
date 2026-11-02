// Opt-in setup of existing frozen packages, never a product install/update path.
"use strict";
if (process.argv.length !== 3 || process.argv[2] !== "--consent" || process.platform !== "win32") {
  throw new Error("Windows lab setup requires exactly --consent");
}
if (process.arch !== "x64") throw new Error("this frozen launcher lab is x64 only");
const fs = require("node:fs"), path = require("node:path"), crypto = require("node:crypto"), cp = require("node:child_process");
const repo = path.resolve(__dirname, "..");
const rows = JSON.parse(fs.readFileSync(path.join(repo, "catalog/windows-launcher-lab.json")));
const pins = JSON.parse(fs.readFileSync(path.join(repo, "catalog/windows-artifacts.json"))).artifacts;
const profile = path.join(repo, ".tools/lab/launcher-profile");
function ordinary(p) {
  let current = path.parse(p).root;
  for (const part of path.relative(current,p).split(path.sep)) {
    current=path.join(current,part);
    if (fs.existsSync(current) && fs.lstatSync(current).isSymbolicLink()) throw new Error("lab reparse path refused");
  }
}
ordinary(profile);
for (const s of ["", "AppData/Roaming", "AppData/Local", "tmp", "npm-cache"]) {
  const target=path.join(profile,s);ordinary(target);fs.mkdirSync(target,{recursive:true});
  if(!fs.realpathSync(target).toLowerCase().startsWith(fs.realpathSync(profile).toLowerCase()+path.sep) && fs.realpathSync(target).toLowerCase()!==fs.realpathSync(profile).toLowerCase())throw new Error("lab output escapes profile");
}
for (const file of ["user.npmrc", "global.npmrc"]) {
  const p=path.join(profile,file); ordinary(p);
  if (!fs.existsSync(p)) fs.writeFileSync(p,"",{flag:"wx"});
  if (fs.statSync(p).size !== 0) throw new Error("lab npm config must be empty");
}
const env={SystemRoot:process.env.SystemRoot,ComSpec:path.join(process.env.SystemRoot,"System32/cmd.exe"),
  PATH:[path.dirname(process.execPath),path.join(process.env.SystemRoot,"System32"),path.join(process.env.SystemRoot,"System32/WindowsPowerShell/v1.0")].join(path.delimiter),PATHEXT:".COM;.EXE;.BAT;.CMD",
  HOME:profile,USERPROFILE:profile,APPDATA:path.join(profile,"AppData/Roaming"),LOCALAPPDATA:path.join(profile,"AppData/Local"),TEMP:path.join(profile,"tmp"),TMP:path.join(profile,"tmp"),
  NPM_CONFIG_OFFLINE:"true",NPM_CONFIG_USERCONFIG:path.join(profile,"user.npmrc"),NPM_CONFIG_GLOBALCONFIG:path.join(profile,"global.npmrc"),NPM_CONFIG_CACHE:path.join(profile,"npm-cache")};
const scripts={"claude-code":"node_modules/@anthropic-ai/claude-code/install.cjs","opencode-v1":"node_modules/opencode-ai/postinstall.mjs","opencode-v2":"node_modules/@opencode/cli/postinstall.mjs"};
const binaries={"codex-cli":"node_modules/@openai/codex-win32-x64/vendor/x86_64-pc-windows-msvc/bin/codex.exe","claude-code":"node_modules/@anthropic-ai/claude-code-win32-x64/claude.exe","opencode-v1":"node_modules/opencode-windows-x64/bin/opencode.exe","opencode-v2":"node_modules/@opencode/cli-windows-x64/bin/opencode.exe"};
function sha(p) {
  ordinary(p);
  if(!fs.statSync(p).isFile() || fs.statSync(p).size > 512*1024*1024) throw new Error("lab file outside bound");
  const fd=fs.openSync(p,"r"), hash=crypto.createHash("sha256"), buffer=Buffer.alloc(65536);
  try {
    const initial=fs.fstatSync(fd);let total=0;
    if(!initial.isFile() || initial.size>512*1024*1024)throw new Error("opened lab file outside bound");
    for (;;) { const n=fs.readSync(fd,buffer,0,buffer.length,null);if(!n)break;total+=n;if(total>512*1024*1024)throw new Error("lab read bound exceeded");hash.update(buffer.subarray(0,n)); }
    if(total!==initial.size || fs.fstatSync(fd).size!==initial.size)throw new Error("lab file changed while hashing");
  }
  finally { fs.closeSync(fd); }
  return hash.digest("hex");
}
for(const row of rows) {
  const pin=pins.find(p=>p.rule_id===row.rule_id && p.tool_id===row.tool_id && p.tool_version===row.version);
  if(!pin || !binaries[row.tool_id]) throw new Error("unknown frozen lab row");
  const root=path.join(repo,".tools/lab/npm-"+row.tool_id);ordinary(root);
  if(sha(path.join(root,binaries[row.tool_id]))!==pin.sha256)throw new Error("dependency does not match frozen pin");
  // Upstream OpenCode setup can select a baseline CPU dependency. Freeze that
  // possible --version target too; it is not a supported process-identity pin.
  if(row.setup_alternative && sha(path.join(root,row.setup_alternative.file))!==row.setup_alternative.sha256)throw new Error("unverified setup alternative");
  if(scripts[row.tool_id]) {
    const script=path.join(root,scripts[row.tool_id]);
    if(sha(script)!==row.setup_script_sha256)throw new Error("unreviewed postinstall script");
    const r=cp.spawnSync(process.execPath,[script],{cwd:path.dirname(script),env,encoding:"utf8",windowsHide:true,timeout:30000});
    if(r.status!==0)throw new Error("offline postinstall failed; no result claimed");
  }
  if(sha(path.join(root,row.entrypoint))!==row.entry_sha256)throw new Error("launcher entry hash mismatch");
  console.log(JSON.stringify({tool:row.tool_id,version:row.version,setup:"PASS"}));
}
