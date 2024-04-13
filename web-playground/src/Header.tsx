import React from 'react';

const Header: React.FC = () => (
  <div className="py-3 pl-4 pr-2 flex items-center">
    <div>
      <h1 className="text-3xl font-bold text-blue-900">🤓 Asciidork Playground</h1>
      <p className="text-sm text-gray-500 antialiased my-0.5 italic">
        Asciidork is an AsciiDoc parser/backend written in Rust and compiled to WASM
      </p>
    </div>
    <a
      href="https://github.com/jaredh159/asciidork"
      className="ml-auto opacity-75 hover:opacity-100"
    >
      <img
        src="https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png"
        className="w-12 h-12"
      />
    </a>
  </div>
);

export default Header;
