const fs = require("fs");
const path = require("path");

const d3Path = "./node_modules/d3/dist/d3.min.js";

if (!fs.existsSync(d3Path)) {
  console.error(
    `FATAL: Could not find the d3 library at its expected path: ${d3Path}`,
  );
  const d3Dir = "./node_modules/d3";
  if (fs.existsSync(d3Dir)) {
    console.error(`Contents of ${d3Dir}:`, fs.readdirSync(d3Dir));
  }
  process.exit(1);
}

const templatePath = "./treemap.template.html";
const outputPath = "./index.html";

console.log("Using D3 from hardcoded path:", d3Path);
console.log("Reading template from:", templatePath);

const d3Content = fs.readFileSync(d3Path, "utf8");
const templateContent = fs.readFileSync(templatePath, "utf8");

const scriptTag = `<script>${d3Content}</script>`;

const finalHtml = templateContent.replace(
  "<!-- D3JS_PLACEHOLDER -->",
  () => scriptTag,
);

fs.writeFileSync(outputPath, finalHtml);

console.log("Successfully created:", outputPath);
