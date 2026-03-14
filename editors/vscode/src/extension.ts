import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
    console.log('sigc extension activated');

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('sigc.compile', compileCommand),
        vscode.commands.registerCommand('sigc.run', runCommand),
        vscode.commands.registerCommand('sigc.explain', explainCommand)
    );

    // Start LSP client if server is configured
    const config = vscode.workspace.getConfiguration('sigc');
    const serverPath = config.get<string>('server.path');

    if (serverPath) {
        startLanguageClient(context, serverPath);
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

function startLanguageClient(context: vscode.ExtensionContext, serverPath: string) {
    const serverOptions: ServerOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio
        },
        debug: {
            command: serverPath,
            transport: TransportKind.stdio
        }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'sigc' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.sig')
        }
    };

    client = new LanguageClient(
        'sigc',
        'sigc Language Server',
        serverOptions,
        clientOptions
    );

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
