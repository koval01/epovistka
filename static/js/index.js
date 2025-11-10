(function() {
    'use strict';

    class PovistkaGenerator {
        constructor() {
            this.init();
        }

        init() {
            this.setupEventListeners();
            this.disableContextMenu();
        }

        setupEventListeners() {
            const form = document.getElementById("generateForm");
            if (form) {
                form.addEventListener("submit", (event) => this.handleSubmit(event));
            }

            // Input field focus/blur handlers
            document.querySelectorAll('input[type="text"]').forEach(input => {
                input.addEventListener('focus', () => this.clearPlaceholder(input));
                input.addEventListener('blur', () => this.restorePlaceholder(input));
            });
        }

        clearPlaceholder(input) {
            input.dataset.placeholder = input.placeholder;
            input.placeholder = '';
        }

        restorePlaceholder(input) {
            if (input.dataset.placeholder) {
                input.placeholder = input.dataset.placeholder;
            }
        }

        disableContextMenu() {
            window.addEventListener('contextmenu', (e) => {
                e.preventDefault();
            });
        }

        async handleSubmit(event) {
            event.preventDefault();

            const formData = new FormData(event.target);
            const jsonData = {};
            formData.forEach((value, key) => {
                jsonData[key] = value.trim();
            });

            // Basic client-side validation
            if (!this.validateInputs(jsonData)) {
                return;
            }

            await this.generateImage(jsonData);
        }

        validateInputs(data) {
            if (!data.name || data.name.trim().length === 0) {
                this.showError("Будь ласка, введіть ім'я");
                return false;
            }
            if (!data.address || data.address.trim().length === 0) {
                this.showError("Будь ласка, введіть адресу");
                return false;
            }
            return true;
        }

        async generateImage(data) {
            try {
                this.showLoading();

                const response = await fetch('/generate', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(data),
                });

                if (!response.ok) {
                    const errorData = await response.json();
                    throw new Error(errorData.error || 'Network response was not ok');
                }

                const blob = await response.blob();
                this.displayImage(blob);

            } catch (error) {
                console.error('Error:', error);
                this.showError(`Помилка: ${error.message}`);
            }
        }

        showLoading() {
            const container = document.getElementById("responseContainer");
            container.innerHTML = '<div class="loading">Генерація повістки...</div>';
        }

        showError(message) {
            const container = document.getElementById("responseContainer");
            container.innerHTML = `<div class="error">${message}</div>`;
        }

        displayImage(blob) {
            const container = document.getElementById("responseContainer");
            const imageUrl = URL.createObjectURL(blob);

            container.innerHTML = `
                <div class="image-result">
                    <img src="${imageUrl}" alt="Згенерована повістка" draggable="false">
                    <div class="image-actions">
                        <button type="button" class="btn-print">Роздрукувати</button>
                        <button type="button" class="btn-save">Зберегти</button>
                    </div>
                </div>
            `;

            // Add event listeners to buttons
            container.querySelector('.btn-print').addEventListener('click', () => this.printImage(imageUrl));
            container.querySelector('.btn-save').addEventListener('click', () => this.saveImage(imageUrl, blob));

            // Scroll to result
            container.scrollIntoView({ behavior: 'smooth', block: 'end' });
        }

        printImage(imageUrl) {
            const printWindow = window.open('', '_blank');
            printWindow.document.write(`
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Друк повістки</title>
                    <style>
                        body { margin: 0; display: flex; justify-content: center; align-items: center; min-height: 100vh; }
                        img { max-width: 100%; height: auto; }
                    </style>
                </head>
                <body>
                    <img src="${imageUrl}" alt="Повістка">
                </body>
                </html>
            `);
            printWindow.document.close();
            printWindow.focus();
            printWindow.print();
        }

        saveImage(imageUrl, blob) {
            const a = document.createElement('a');
            a.href = imageUrl;
            a.download = `povistka_${this.generateRandomHex(8)}.png`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);

            // Clean up URL
            setTimeout(() => URL.revokeObjectURL(imageUrl), 1000);
        }

        generateRandomHex(length) {
            return Array.from(crypto.getRandomValues(new Uint8Array(length)))
                .map(b => b.toString(16).padStart(2, '0'))
                .join('');
        }
    }

    // Initialize the application when DOM is loaded
    document.addEventListener('DOMContentLoaded', () => {
        new PovistkaGenerator();
    });

})();