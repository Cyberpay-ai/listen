export function ChatContainer({
  children,
  inputMessage,
}: {
  children: React.ReactNode;
  inputMessage: string;
}) {
  return (
    <div className="flex flex-col gap-4 h-[85vh] w-full font-mono">
       <div className="flex justify-between items-center lg:px-4 py-8">
        <h1 className="lg:text-2xl text-xl font-bold text-white lg:text-left text-center">
          Chatbot
        </h1>
      </div>
      <div className="flex-1 overflow-hidden">
        <div className="h-full border-2 border-purple-500/30 rounded-lg overflow-hidden bg-black/40 backdrop-blur-sm">
          <div className="h-full flex flex-col">
            <div className="flex-1 overflow-y-auto p-4 scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
              {children}
            </div>
          </div>
        </div>
      </div>
      <ChatInput inputMessage={inputMessage} />
    </div>
  );
}

export function ChatInput({ inputMessage }: { inputMessage: string }) {
  return (
    <div className="min-h-12 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm px-3 py-3">
      <div className="inline">
        <span className="text-white whitespace-pre-wrap break-words">
          {inputMessage}
        </span>
        <span className="w-2 h-5 bg-white terminal-blink ml-[1px] inline-block align-middle" />
      </div>
    </div>
  );
}
