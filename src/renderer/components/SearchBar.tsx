import { useRef, useEffect } from 'react';
import { Search, X } from 'lucide-react';

interface SearchBarProps {
  /**
   * The current query. Fully controlled: HomePage parks the search text in
   * the URL so the browsing trail restores it, and a mirrored copy in here
   * would be a second source of truth free to drift out of sync with it.
   */
  value: string;
  onSearch: (query: string) => void;
  placeholder?: string;
}

function SearchBar({ value, onSearch, placeholder = 'Search…' }: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  // The listener only ever needs the LATEST onSearch, and its identity now
  // changes on every keystroke (it writes through the query string). Holding
  // it in a ref keeps the window listener registered once instead of being
  // torn down and rebuilt per character.
  const onSearchRef = useRef(onSearch);
  useEffect(() => {
    onSearchRef.current = onSearch;
  }, [onSearch]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const isTyping = target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA');

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        inputRef.current?.focus();
      }
      if (e.key === '/' && !isTyping) {
        e.preventDefault();
        inputRef.current?.focus();
      }
      if (e.key === 'Escape' && document.activeElement === inputRef.current) {
        onSearchRef.current('');
        inputRef.current?.blur();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onSearch(e.target.value);
  };

  const handleClear = () => {
    onSearch('');
    inputRef.current?.focus();
  };

  return (
    <div className="library-search">
      <span className="library-search-icon"><Search size={16} /></span>
      <input
        ref={inputRef}
        type="text"
        className="library-search-input"
        placeholder={placeholder}
        value={value}
        onChange={handleChange}
        spellCheck={false}
        autoComplete="off"
      />
      {value ? (
        <button
          className="library-search-clear"
          onClick={handleClear}
          aria-label="Clear search"
        >
          <X size={14} />
        </button>
      ) : (
        <span className="library-search-hint chip chip--sm" aria-hidden="true">Ctrl K</span>
      )}
    </div>
  );
}

export default SearchBar;
