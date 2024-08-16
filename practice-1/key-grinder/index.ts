import { Keypair } from '@solana/web3.js';
import bs58 from 'bs58';

// Функція для генерації приватного ключа з певним префіксом (нечутливим до регістру) у Base58
function generatePrivateKeyWithPrefix(prefix: string): Uint8Array | null {
    const lowerCasePrefix = prefix.toLowerCase();
    let attempts = 0;

    while (true) {
        const keypair = Keypair.generate();
        const privateKeyBase58 = bs58.encode(keypair.secretKey).toLowerCase();
        attempts++;

        // Виведення кількості спроб кожні 100 000
        if (attempts % 100000 === 0) {
            console.log(`Спроба: ${attempts}`);
        }

        // Перевіряємо, чи починається приватний ключ з заданого префіксу (нечутливого до регістру)
        if (privateKeyBase58.startsWith(lowerCasePrefix)) {
            console.log(`Кількість спроб: ${attempts}`);
            return keypair.secretKey;
        }
    }
}

// Встановіть префікс (нечутливий до регістру)
const prefix = 'alex';

const privateKey = generatePrivateKeyWithPrefix(prefix);

if (privateKey) {
    console.log(`Private Key (Base58): ${bs58.encode(privateKey)}`);
} else {
    console.log('Не вдалося знайти приватний ключ з таким префіксом.');
}
