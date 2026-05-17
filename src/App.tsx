import { format } from "date-fns";
import { AcornLogo } from "@/components/acorn-logo";

function App() {
  const today = format(new Date(), "EEEE, MMMM d");

  return (
    <main className="min-h-screen flex items-center justify-center bg-background text-foreground">
      <div className="flex flex-col items-center gap-4">
        <AcornLogo size={64} />
        <h1 className="text-2xl font-medium tracking-tight">Acorn</h1>
        <p className="text-sm text-muted-foreground">{today}</p>
        <p className="text-xs text-muted-foreground mt-6 max-w-xs text-center">
          Stash your day, one acorn at a time.
        </p>
      </div>
    </main>
  );
}

export default App;
