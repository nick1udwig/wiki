import React, { useState, useEffect } from 'react';
import { wikiApi, WikiInvite } from '../api/wiki';
import './InviteCenter.css';

export function InviteCenter() {
  const [invites, setInvites] = useState<WikiInvite[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadInvites = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const myInvites = await wikiApi.listMyInvites();
      setInvites(myInvites);
    } catch (err: any) {
      setError(err.message || 'Failed to load invites');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadInvites();
  }, []);

  const handleRespond = async (inviteId: string, accept: boolean) => {
    try {
      await wikiApi.respondToInvite(inviteId, accept);
      // Reload invites to update the list
      await loadInvites();
    } catch (err: any) {
      setError(err.message || 'Failed to respond to invite');
    }
  };

  if (isLoading) {
    return <div className="invite-center loading">Loading invites...</div>;
  }

  return (
    <div className="invite-center">
      <h3>Pending Invites</h3>
      
      {error && (
        <div className="error-message">
          {error}
        </div>
      )}
      
      {invites.length === 0 ? (
        <div className="empty-invites">
          <p>No pending invites</p>
        </div>
      ) : (
        <div className="invite-list">
          {invites.map((invite) => (
            <div key={invite.id} className={`invite-card ${invite.is_expired ? 'expired' : ''}`}>
              <div className="invite-info">
                <h4>{invite.wiki_name}</h4>
                <p className="invite-meta">
                  Invited by <strong>{invite.inviter_id}</strong>
                </p>
                <p className="invite-date">
                  {new Date(invite.created_at).toLocaleDateString()}
                </p>
                {invite.is_expired && (
                  <p className="invite-expired">This invite has expired</p>
                )}
              </div>
              
              {!invite.is_expired && (
                <div className="invite-actions">
                  <button 
                    className="accept-btn"
                    onClick={() => handleRespond(invite.id, true)}
                  >
                    Accept
                  </button>
                  <button 
                    className="decline-btn"
                    onClick={() => handleRespond(invite.id, false)}
                  >
                    Decline
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}