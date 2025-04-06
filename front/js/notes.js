document.addEventListener('DOMContentLoaded', () => {
    const notesList = document.getElementById('notes-list');
    const noteEditor = document.getElementById('note-editor');
    const newNoteBtn = document.getElementById('new-note-btn');
    const saveNoteBtn = document.getElementById('save-note-btn');
    const cancelEditBtn = document.getElementById('cancel-edit-btn');
    const deleteNoteBtn = document.getElementById('delete-note-btn');
    const logoutBtn = document.getElementById('logout-btn');
    const searchInput = document.getElementById('search-notes');
    
    let currentNoteId = null;
    let notes = [];
    
    // Verificar autenticación
    const token = localStorage.getItem('authToken');
    if (!token && !window.location.pathname.endsWith('auth.html')) {
        window.location.href = 'auth.html';
        return;
    }
    
    // Cargar notas al iniciar
    loadNotes();
    
    // Buscar notas
    searchInput.addEventListener('input', () => {
        const searchTerm = searchInput.value.toLowerCase();
        renderNotes(notes.filter(note => 
            note.title.toLowerCase().includes(searchTerm) || 
            note.content.toLowerCase().includes(searchTerm)
        ));
    });
    
    // Nueva nota
    newNoteBtn.addEventListener('click', () => {
        currentNoteId = null;
        document.getElementById('note-title').value = '';
        document.getElementById('note-content').value = '';
        deleteNoteBtn.classList.add('hidden');
        noteEditor.classList.remove('hidden');
        document.getElementById('note-title').focus();
    });
    
    // Guardar nota
    saveNoteBtn.addEventListener('click', async () => {
        const title = document.getElementById('note-title').value.trim();
        const content = document.getElementById('note-content').value.trim();
        
        if (!title) {
            alert('El título es requerido');
            return;
        }
        
        try {
            if (currentNoteId) {
                // Actualizar nota existente
                await updateNote(currentNoteId, title, content);
            } else {
                // Crear nueva nota
                await createNote(title, content);
            }
            
            noteEditor.classList.add('hidden');
            loadNotes();
        } catch (error) {
            console.error('Error al guardar la nota:', error);
            alert('Error al guardar la nota');
        }
    });
    
    // Cancelar edición
    cancelEditBtn.addEventListener('click', () => {
        noteEditor.classList.add('hidden');
    });
    
    // Eliminar nota
    deleteNoteBtn.addEventListener('click', async () => {
        if (!currentNoteId || !confirm('¿Estás seguro de eliminar esta nota?')) {
            return;
        }
        
        try {
            await deleteNote(currentNoteId);
            noteEditor.classList.add('hidden');
            loadNotes();
        } catch (error) {
            console.error('Error al eliminar la nota:', error);
            alert('Error al eliminar la nota');
        }
    });
    
    // Cerrar sesión
    logoutBtn.addEventListener('click', () => {
        localStorage.removeItem('authToken');
        window.location.href = 'auth.html';
    });
    
    // Funciones para manejar las notas
    async function loadNotes() {
        try {
            const response = await fetch('/api/auth/login', {
                headers: {
                    'Authorization': `Bearer ${token}`,
                },
            });
            
            if (!response.ok) {
                if (response.status === 401) {
                    localStorage.removeItem('authToken');
                    window.location.href = 'auth.html';
                }
                throw new Error('Error al cargar las notas');
            }
            
            notes = await response.json();
            renderNotes(notes);
        } catch (error) {
            console.error('Error al cargar notas:', error);
            alert('Error al cargar las notas');
        }
    }
    
    function renderNotes(notesToRender) {
        notesList.innerHTML = '';
        
        if (notesToRender.length === 0) {
            notesList.innerHTML = '<p class="no-notes">No hay notas disponibles</p>';
            return;
        }
        
        notesToRender.forEach(note => {
            const noteElement = document.createElement('div');
            noteElement.className = 'note-card';
            noteElement.innerHTML = `
                <h3>${note.title}</h3>
                <p>${note.content}</p>
                <p class="note-date">${formatDate(note.updated_at)}</p>
            `;
            
            noteElement.addEventListener('click', () => {
                openNoteForEdit(note);
            });
            
            notesList.appendChild(noteElement);
        });
    }
    
    function openNoteForEdit(note) {
        currentNoteId = note.id;
        document.getElementById('note-title').value = note.title;
        document.getElementById('note-content').value = note.content;
        deleteNoteBtn.classList.remove('hidden');
        noteEditor.classList.remove('hidden');
    }
    
    async function createNote(title, content) {
        const response = await fetch('https://localhost:8443/api/notes', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`,
            },
            body: JSON.stringify({ title, content }),
        });
        
        if (!response.ok) {
            throw new Error('Error al crear la nota');
        }
    }
    
    async function updateNote(id, title, content) {
        const response = await fetch(`https://localhost:8443/api/notes/${id}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`,
            },
            body: JSON.stringify({ title, content }),
        });
        
        if (!response.ok) {
            throw new Error('Error al actualizar la nota');
        }
    }
    
    async function deleteNote(id) {
        const response = await fetch(`https://localhost:8443/api/notes/${id}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${token}`,
            },
        });
        
        if (!response.ok) {
            throw new Error('Error al eliminar la nota');
        }
    }
    
    function formatDate(dateString) {
        const date = new Date(dateString);
        return date.toLocaleDateString('es-ES', {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
        });
    }
});