use rand::seq::SliceRandom;
use rand::thread_rng;

#[allow(dead_code)]
const STOIC_QUOTES: &[&str] = &[
    // Marcus Aurelius (Meditations)
    "You have power over your mind - not outside events. Realize this, and you will find strength. -- Marcus Aurelius",
    "The impediment to action advances action. What stands in the way becomes the way. -- Marcus Aurelius",
    "Very little is needed to make a happy life; it is all within yourself. -- Marcus Aurelius",
    "The best revenge is to be unlike him who performed the injury. -- Marcus Aurelius",
    "Waste no more time arguing about what a good man should be. Be one. -- Marcus Aurelius",
    "If it is not right, do not do it. If it is not true, do not say it. -- Marcus Aurelius",
    "The soul becomes dyed with the color of its thoughts. -- Marcus Aurelius",
    "Accept the things to which fate binds you, and love the people with whom fate brings you together. -- Marcus Aurelius",
    "Dwell on the beauty of life. Watch the stars, and see yourself running with them. -- Marcus Aurelius",
    "Our life is what our thoughts make it. -- Marcus Aurelius",

    // Seneca (Letters, Essays)
    "We suffer more often in imagination than in reality. -- Seneca",
    "Luck is what happens when preparation meets opportunity. -- Seneca",
    "It is not the man who has too little, but the man who craves more, that is poor. -- Seneca",
    "Difficulties strengthen the mind, as labor does the body. -- Seneca",
    "Life is long if you know how to use it. -- Seneca",
    "Begin at once to live, and count each separate day as a separate life. -- Seneca",
    "True happiness is to enjoy the present, without anxious dependence upon the future. -- Seneca",
    "He who is brave is free. -- Seneca",

    // Epictetus (Discourses, Enchiridion)
    "It's not what happens to you, but how you react to it that matters. -- Epictetus",
    "First say to yourself what you would be; and then do what you have to do. -- Epictetus",
    "Wealth consists not in having great possessions, but in having few wants. -- Epictetus",
    "No man is free who is not master of himself. -- Epictetus",
    "He is a wise man who does not grieve for the things which he has not, but rejoices for those which he has. -- Epictetus",

    // Other Stoics
    "The willing are led by fate, the reluctant dragged. -- Cleanthes",
    "To live is not merely to breathe; it is to act. -- Rousseau (Stoic-influenced)",
    "The only wealth which you will keep forever is the wealth you have given away. -- Marcus Aurelius",
];

/// Returns a random Stoic quote from the embedded collection
#[allow(dead_code)]
pub fn random_stoic_quote() -> &'static str {
    let mut rng = thread_rng();
    STOIC_QUOTES.choose(&mut rng).unwrap_or(&STOIC_QUOTES[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_quote_returns_nonempty() {
        let quote = random_stoic_quote();
        assert!(!quote.is_empty());
        assert!(quote.contains("--"));
    }

    #[test]
    fn test_quotes_array_not_empty() {
        assert!(STOIC_QUOTES.len() >= 20);
    }
}
