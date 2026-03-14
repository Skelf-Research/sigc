"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    console.log('sigc extension activated');
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('sigc.compile', compileCommand), vscode.commands.registerCommand('sigc.run', runCommand), vscode.commands.registerCommand('sigc.explain', explainCommand));
    // Start LSP client if server is configured
    const config = vscode.workspace.getConfiguration('sigc');
    const serverPath = config.get('server.path');
    if (serverPath) {
        startLanguageClient(context, serverPath);
    }
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
function startLanguageClient(context, serverPath) {
    const serverOptions = {
        run: {
            command: serverPath,
            transport: node_1.TransportKind.stdio
        },
        debug: {
            command: serverPath,
            transport: node_1.TransportKind.stdio
        }
    };
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'sigc' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.sig')
        }
    };
    client = new node_1.LanguageClient('sigc', 'sigc Language Server', serverOptions, clientOptions);
    client.start();
}
async function compileCommand() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }
    const document = editor.document;
    if (document.languageId !== 'sigc') {
        vscode.window.showErrorMessage('Not a sigc file');
        return;
    }
    const filePath = document.fileName;
    const terminal = vscode.window.createTerminal('sigc');
    terminal.show();
    terminal.sendText(`sigc compile "${filePath}"`);
}
async function runCommand() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }
    const document = editor.document;
    if (document.languageId !== 'sigc') {
        vscode.window.showErrorMessage('Not a sigc file');
        return;
    }
    // Save before running
    await document.save();
    const filePath = document.fileName;
    const terminal = vscode.window.createTerminal('sigc');
    terminal.show();
    terminal.sendText(`sigc run "${filePath}"`);
}
async function explainCommand() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }
    const document = editor.document;
    if (document.languageId !== 'sigc') {
        vscode.window.showErrorMessage('Not a sigc file');
        return;
    }
    const filePath = document.fileName;
    const terminal = vscode.window.createTerminal('sigc');
    terminal.show();
    terminal.sendText(`sigc explain "${filePath}"`);
}
//# sourceMappingURL=extension.js.map