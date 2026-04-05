const fs = require('fs');
const { createCanvas } = require('canvas');

const canvas = createCanvas(32, 32);
const ctx = canvas.getContext('2d');

// 绘制蓝色背景
ctx.fillStyle = '#3B82F6';
ctx.fillRect(0, 0, 32, 32);

// 绘制白色文字 "AI"
ctx.fillStyle = '#FFFFFF';
ctx.font = 'bold 16px Arial';
ctx.textAlign = 'center';
ctx.textBaseline = 'middle';
ctx.fillText('AI', 16, 16);

const buffer = canvas.toBuffer('image/png');
fs.writeFileSync('D:\\program\\kiroOne\\src-tauri\\app-icon.png', buffer);
