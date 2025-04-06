document.addEventListener('DOMContentLoaded', () => {
    const loginForm = document.getElementById('login-form');
    const registerForm = document.getElementById('register-form');
    const tabButtons = document.querySelectorAll('.tab-btn');
    const loginError = document.getElementById('login-error');
    const registerError = document.getElementById('register-error');
    
    // Cambiar entre pestañas de login y registro
    tabButtons.forEach(button => {
        button.addEventListener('click', () => {
            const tab = button.getAttribute('data-tab');
            
            // Actualizar botones activos
            tabButtons.forEach(btn => btn.classList.remove('active'));
            button.classList.add('active');
            
            // Mostrar el formulario correspondiente
            if (tab === 'login') {
                loginForm.classList.remove('hidden');
                registerForm.classList.add('hidden');
            } else {
                loginForm.classList.add('hidden');
                registerForm.classList.remove('hidden');
            }
        });
    });
    
    // Manejar login
    loginForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        loginError.textContent = '';
        
        const username = document.getElementById('login-username').value;
        const password = document.getElementById('login-password').value;
        
        try {
            const response = await fetch('https://localhost:8443/api/auth/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ username, password }),
            });
            
            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Error al iniciar sesión');
            }
            
            const token = await response.text();
            localStorage.setItem('authToken', token);
            window.location.href = 'index.html';
        } catch (error) {
            loginError.textContent = error.message;
            console.error('Login error:', error);
        }
    });
    
    // Manejar registro
    registerForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        registerError.textContent = '';
        
        const username = document.getElementById('register-username').value;
        const password = document.getElementById('register-password').value;
        const confirmPassword = document.getElementById('register-confirm-password').value;
        
        if (password !== confirmPassword) {
            registerError.textContent = 'Las contraseñas no coinciden';
            return;
        }
        
        try {
            const response = await fetch('/api/auth/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ username, password }),
            });
            
            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Error al registrarse');
            }
            
            // Después de registrarse, cambia a la pestaña de login
            document.querySelector('[data-tab="login"]').click();
            document.getElementById('login-username').value = username;
            registerError.textContent = '';
        } catch (error) {
            registerError.textContent = error.message;
            console.error('Register error:', error);
        }
    });
    
    // Si ya está autenticado, redirigir a la página principal
    if (localStorage.getItem('authToken') && window.location.pathname.endsWith('auth.html')) {
        window.location.href = 'index.html';
    }
});