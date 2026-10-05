// Prints a PSD's tree, masks and resource keys through ag-psd.
const { readPsd } = require('ag-psd');
const fs = require('fs');
const psd = readPsd(fs.readFileSync(process.argv[2]), { skipCompositeImageData: true, skipLayerImageData: true, skipThumbnail: true });
console.log(`ag-psd ${psd.width}x${psd.height} channels=${psd.channels} bits=${psd.bitsPerChannel} mode=${psd.colorMode} icc=${!!(psd.imageResources || {}).iccProfile}`);
function walk(ch, ind = '  ') {
  for (const l of ch) {
    let line = `${ind}'${l.name}' opacity=${l.opacity} blend=${l.blendMode} bbox=${l.left},${l.top},${l.right},${l.bottom}`;
    if (l.mask) line += ` mask ${l.mask.left},${l.mask.top},${l.mask.right},${l.mask.bottom} default=${l.mask.defaultColor} disabled=${!!l.mask.disabled}`;
    console.log(line);
    if (l.children) walk(l.children, ind + '  ');
  }
}
walk(psd.children);
console.log('resources:', Object.keys(psd.imageResources || {}).join(' '));
