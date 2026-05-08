import { createContext, useContext, useEffect, useState, useCallback, type ReactNode } from "react";

export type TitleLanguage = "JP" | "EN";

interface Ctx {
  lang: TitleLanguage;
  setLang: (next: TitleLanguage) => void;
  /**
   * Pick the appropriate title given the user's language choice. JP →
   * romaji (most folders are already romaji-named, so this is the
   * "default" feel). EN → english localization, with romaji as fallback
   * when no English title is available, and folder name as ultimate
   * fallback when nothing has been matched yet.
   */
  pickTitle: (opts: { titleRomaji?: string | null; titleEnglish?: string | null; folderName: string }) => string;
}

const TitleLanguageContext = createContext<Ctx | null>(null);

const STORAGE_KEY = "anibeam.titleLanguage";

export function TitleLanguageProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<TitleLanguage>(() => {
    if (typeof window === "undefined") return "JP";
    const saved = window.localStorage.getItem(STORAGE_KEY);
    return saved === "EN" ? "EN" : "JP";
  });

  useEffect(() => {
    window.localStorage.setItem(STORAGE_KEY, lang);
  }, [lang]);

  const setLang = useCallback((next: TitleLanguage) => setLangState(next), []);

  const pickTitle = useCallback<Ctx["pickTitle"]>(
    ({ titleRomaji, titleEnglish, folderName }) => {
      if (lang === "EN") {
        return titleEnglish || titleRomaji || folderName;
      }
      return titleRomaji || titleEnglish || folderName;
    },
    [lang],
  );

  return (
    <TitleLanguageContext.Provider value={{ lang, setLang, pickTitle }}>
      {children}
    </TitleLanguageContext.Provider>
  );
}

export function useTitleLanguage(): Ctx {
  const ctx = useContext(TitleLanguageContext);
  if (!ctx) {
    throw new Error("useTitleLanguage must be used inside TitleLanguageProvider");
  }
  return ctx;
}
