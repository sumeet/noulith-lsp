const vscode = require('vscode');
const { LanguageClient, LanguageClientOptions, ExitNotification } = require('vscode-languageclient');

function activate(context) {
    let serverOptions = {
        command: '/home/sumeet/noulith-lsp/target/debug/noulith-lsp',
        args: []
    };  
    let clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'noulith' }],
        synchronize: {
            configurationSection: 'noulith',
            fileEvents: vscode.workspace.createFileSystemWatcher('**/.noul')
        }
    };
    let client = new LanguageClient('noulith', 'nlsp', serverOptions, clientOptions);
    client.start();
}

function deactivate() {
}


module.exports = {
    activate,
    deactivate
}
